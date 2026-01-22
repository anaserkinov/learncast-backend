use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow)]
pub struct SessionEntity {
    pub id: i64,
    pub user_id: i64,
    pub refresh_token_hash: String,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime
}
