use smart_default::SmartDefault;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow, SmartDefault)]
pub struct SnipEntity {
    pub id: i64,
    pub client_snip_id: String,
    pub author_id: i64,
    pub topic_id: Option<i64>,
    pub lesson_id: i64,
    pub user_id: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub note_text: Option<String>,
    #[default(OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime
}

#[derive()]
pub struct SnipInput {
    pub client_snip_id: String,
    pub author_id: i64,
    pub topic_id: Option<i64>,
    pub lesson_id: i64,
    pub user_id: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub note_text: Option<String>,
    pub created_at: OffsetDateTime
}

#[derive(Debug, FromRow, SmartDefault)]
pub struct SnipEntityWithLesson {
    #[sqlx(flatten)]
    pub snip: SnipEntity,
    //lesson
    pub lesson_title: String,
    pub lesson_description: Option<String>,
    pub lesson_cover_image_path: Option<String>,
    pub lesson_audio_path: String,
    pub lesson_duration: i64,
    pub lesson_file_size: i64,
    pub lesson_listen_count: i64,
    pub lesson_snip_count: i64,
    #[default(OffsetDateTime::now_utc())]
    pub lesson_created_at: OffsetDateTime,
    //author
    pub author_name: String,
    pub author_avatar_path: Option<String>,
    #[default(OffsetDateTime::now_utc())]
    pub author_created_at: OffsetDateTime,
    pub author_lesson_count: i64,
    //topic
    pub topic_title: Option<String>,
    pub topic_description: Option<String>,
    pub topic_cover_image_path: Option<String>,
    pub topic_created_at: Option<OffsetDateTime>,
    pub topic_lesson_count: Option<i64>,
    pub topic_total_duration: Option<i64>,
    pub topic_snip_count: Option<i64>,
}