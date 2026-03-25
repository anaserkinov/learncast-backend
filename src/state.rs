use std::sync::Arc;
use redis::Client;
use sqlx::PgPool;
use aws_sdk_s3 as s3;
use crate::utils::telegram::JwksCache;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis_client: Client,
    pub s3_client: s3::Client,
    pub jwks_cache: Arc<JwksCache>
}

impl AppState {
    pub fn new(
        db: PgPool,
        redis_client: Client,
        s3_client: s3::Client,
        jwks_cache: Arc<JwksCache>
    ) -> Self {
        Self { db, redis_client, s3_client, jwks_cache }
    }
}
