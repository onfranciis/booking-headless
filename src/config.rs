use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub port: u16,
    pub gcs_bucket_name: String,
    pub google_service_account_path: String, // Path to the downloaded JSON key
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let google_service_account_path = expect_env("GOOGLE_SERVICE_ACCOUNT_PATH");

        unsafe {
            std::env::set_var(
                "GOOGLE_APPLICATION_CREDENTIALS",
                &google_service_account_path,
            );
        }

        let port = get_env_or_default("PORT", "8080")
            .parse::<u16>()
            .expect("PORT must be a valid number");

        Self {
            database_url: expect_env("DATABASE_URL"),
            jwt_secret: expect_env("JWT_SECRET"),
            google_client_id: expect_env("GOOGLE_CLIENT_ID"),
            google_client_secret: expect_env("GOOGLE_CLIENT_SECRET"),
            google_redirect_uri: expect_env("GOOGLE_REDIRECT_URI"),
            gcs_bucket_name: expect_env("GCS_BUCKET_NAME"),
            redis_url: expect_env("REDIS_URL"),
            google_service_account_path,
            port,
        }
    }
}

// Helper function to panic with a clear error
fn expect_env(var_name: &str) -> String {
    env::var(var_name).unwrap_or_else(|_| panic!("Missing required env variable: {}", var_name))
}

fn get_env_or_default(var_name: &str, default: &str) -> String {
    env::var(var_name).unwrap_or_else(|_| default.to_string())
}
