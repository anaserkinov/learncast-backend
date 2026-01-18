use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow)]
pub struct UserEntity {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub avatar_path: Option<String>,
    pub email: Option<String>,
    pub telegram_id: Option<i64>,
    pub telegram_username: Option<String>,
    pub google_id: Option<String>,
    pub password_hash: Option<String>,
    pub is_admin: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime
}
