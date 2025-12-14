use crate::config::Config;
use crate::routes::utils_routes::{bad_request_response, internal_server_error_response};
use crate::structs::db_struct::{Auth, GoogleCode, GoogleUserInfo, TokenClaims, User};
use crate::structs::response_struct::ApiResponse;
use actix_web::{HttpResponse, Responder, web};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
};
use reqwest;
use sqlx::PgPool;

// Helper to create the OAuth client
fn create_google_oauth_client(
    config: Config,
) -> oauth2::Client<
    oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    oauth2::StandardTokenIntrospectionResponse<
        oauth2::EmptyExtraTokenFields,
        oauth2::basic::BasicTokenType,
    >,
    oauth2::StandardRevocableToken,
    oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
> {
    let Config {
        google_client_id,
        google_client_secret,
        google_redirect_uri,
        ..
    } = config.clone();
    let google_client_id = ClientId::new(google_client_id);
    let google_client_secret = ClientSecret::new(google_client_secret);
    let redirect_uri = RedirectUrl::new(google_redirect_uri).expect("Invalid redirect URL");
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid auth URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v4/token".to_string())
        .expect("Invalid token URL");

    BasicClient::new(google_client_id)
        .set_client_secret(google_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_uri)
}

async fn google_auth_handler(
    config: web::Data<Config>,
    pool: web::Data<PgPool>,
    body: web::Json<GoogleCode>,
    injected_http_client: web::Data<reqwest::Client>,
) -> impl Responder {
    println!("Received Google auth code: {}", body.code);
    let oauth_client = create_google_oauth_client(config.get_ref().clone());
    println!("OAuth client created.");
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");
    println!("HTTP client created.");

    // Exchange the code from the frontend for a token
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(body.code.clone()))
        .request_async(&http_client)
        .await;
    println!("Token exchange completed.");

    let token = match token_result {
        Ok(token) => token,
        Err(e) => return internal_server_error_response(format!("Token exchange failed: {}", e)),
    };

    println!("Access token obtained.");

    let access_token = token.access_token().secret();
    println!("Access Token: {}", access_token);

    // Get the refresh token (only provided on first login/consent)
    let refresh_token = token.refresh_token().map(|t| t.secret().to_string());
    println!(
        "Refresh Token: {}",
        refresh_token.as_deref().unwrap_or("None")
    );

    // Use the access token to get the user's info from Google
    let user_info_res = injected_http_client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await;

    println!("User info request sent");
    println!("User info: {:#?}", user_info_res);

    let user_info: GoogleUserInfo = match user_info_res {
        Ok(res) => match res.json().await {
            Ok(info) => info,
            Err(e) => {
                return internal_server_error_response(format!("Failed to parse user info: {}", e));
            }
        },
        Err(e) => return internal_server_error_response(format!("Failed to get user info: {}", e)),
    };

    println!(
        "User email obtained: {:?}, Google ID: {}, Name: {}",
        user_info.email,
        user_info.sub,
        user_info.name.as_deref().unwrap_or("None")
    );

    // Start a database transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    println!("Database transaction started.");

    let email = user_info
        .email
        .unwrap_or_else(|| format!("user_{}@example.com", user_info.sub));

    let name = user_info.name.unwrap_or_else(|| "My Business".to_string());

    // Find or Create the User (based on email)
    // This query finds a user by email. If they don't exist, it creates them.
    let user = match sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, username, business_name)
        VALUES ($1, $2, $3)
        ON CONFLICT (email) DO UPDATE 
        SET email = EXCLUDED.email -- This ensures we get the row back
        RETURNING *
        "#,
        email,
        email.clone(),
        name
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(user) => user,

        Err(sqlx::Error::Database(db_err)) => {
            tx.rollback().await.ok();
            if db_err.is_unique_violation() {
                return bad_request_response("That username is already taken.".to_string());
            }

            return internal_server_error_response(db_err.to_string());
        }

        Err(e) => {
            tx.rollback().await.ok();
            return internal_server_error_response(format!("Failed to upsert user: {}", e));
        }
    };

    // Find or Create the Auth entry (based on Google ID)
    // This links their Google account to their user.id.
    let _auth_record = match sqlx::query_as!(
        Auth,
        r#"
        INSERT INTO auth (user_id, google_id, refresh_token)
        VALUES ($1, $2, $3)
        ON CONFLICT (google_id) DO UPDATE
        SET refresh_token = COALESCE($3, auth.refresh_token), user_id = $1
        RETURNING *
        "#,
        user.id,
        user_info.sub, // The unique Google ID
        refresh_token  // The new refresh token
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(auth_record) => auth_record,

        Err(e) => {
            tx.rollback().await.ok();
            return internal_server_error_response(format!("Failed to upsert auth: {}", e));
        }
    };

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        return internal_server_error_response(e.to_string());
    }

    // Create and issue our API's JWT
    let jwt_secret = &config.jwt_secret.clone();
    let now = Utc::now();
    let claims = TokenClaims {
        sub: user.id,
        exp: (now + Duration::days(7)).timestamp(),
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    let response = ApiResponse {
        success: true,
        data: Some(token),
        message: Some("Authentication successful".to_string()),
    };
    HttpResponse::Ok().json(response)
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

pub fn auth_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route("/google/connect", web::post().to(google_auth_handler)));
}
