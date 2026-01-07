use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret_refresh: String,
    pub jwt_secret_access: String,
    pub telegram_bot_token: String,
    pub r2_endpoint_url: String,
    pub r2_bucket_name: String
}

impl AppConfig {
    pub fn load() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL missing"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
            jwt_secret_refresh: env::var("JWT_SECRET_REFRESH").expect("JWT_SECRET_REFRESH missing"),
            jwt_secret_access: env::var("JWT_SECRET_ACCESS").expect("JWT_SECRET_ACCESS missing"),
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN missing"),
            r2_endpoint_url: env::var("R2_ENDPOINT_URL").expect("R2_ENDPOINT_URL missing"),
            r2_bucket_name: env::var("R2_BUCKET_NAME").expect("R2_BUCKET_NAME missing")
        }
    }
}
