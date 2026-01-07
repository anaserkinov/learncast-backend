use smart_default::SmartDefault;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow, SmartDefault)]
pub struct AuthorEntity {
    pub id: i64,
    pub name: String,
    pub avatar_path: Option<String>,
    #[default(OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime,
    pub lesson_count: i64
}

#[derive()]
pub struct AuthorInput {
    pub name: String,
    pub avatar_path: Option<String>
}