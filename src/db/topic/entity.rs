use smart_default::SmartDefault;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow, SmartDefault)]
pub struct TopicEntity {
    pub id: i64,
    pub author_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    #[default(OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime,
    pub lesson_count: i64,
    pub total_duration: i64,
    pub snip_count: i64
}

#[derive(Debug, FromRow, SmartDefault)]
#[sqlx(default)]
pub struct TopicWithAuthor {
    #[sqlx(flatten)]
    pub topic: TopicEntity,
    pub author_name: String,
    pub author_avatar_path: Option<String>,
    #[default(OffsetDateTime::now_utc())]
    pub author_created_at: OffsetDateTime,
    pub author_lesson_count: i64,
    pub completed_lesson_count: Option<i64>
}

#[derive()]
pub struct TopicInput {
    pub author_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>
}