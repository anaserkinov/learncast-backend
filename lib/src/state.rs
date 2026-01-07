use redis::Client;
use sqlx::PgPool;
use aws_sdk_s3 as s3;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis_client: Client,
    pub s3_client: s3::Client
}

impl AppState {
    pub fn new(
        db: PgPool,
        redis_client: Client,
        s3_client: s3::Client
    ) -> Self {
        Self { db, redis_client, s3_client }
    }
}
