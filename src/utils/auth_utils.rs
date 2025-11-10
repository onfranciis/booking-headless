use crate::config::Config;
use actix_web::web;

pub async fn get_new_access_token(
    config: web::Data<Config>,
    client: &reqwest::Client,
    refresh_token: String,
) -> Result<String, String> {
    let params = [
        ("client_id", &config.google_client_id),
        ("client_secret", &config.google_client_secret),
        ("refresh_token", &refresh_token),
        ("grant_type", &"refresh_token".to_string()),
    ];

    let res = client
        .post("https://www.googleapis.com/oauth2/v4/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let error_text = res.text().await.unwrap_or_default();
        return Err(format!("Google token refresh failed: {}", error_text));
    }

    let token_res: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    token_res["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to parse access_token from Google".to_string())
}
