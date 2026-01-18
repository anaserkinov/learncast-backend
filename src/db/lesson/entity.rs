use crate::module::common::enums::UserProgressStatus;
use smart_default::SmartDefault;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive()]
pub struct LessonInput {
    pub author_id: i64,
    pub topic_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    pub audio_path: String,
    pub duration: i64,
    pub file_size: i64
}

#[derive(Debug, FromRow, SmartDefault)]
pub struct LessonEntity {
    pub id: i64,
    pub author_id: i64,
    pub topic_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    pub audio_path: String,
    pub duration: i64,
    pub file_size: i64,
    pub listen_count: i64,
    pub snip_count: i64,
    #[default(OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime
}

#[derive(FromRow, SmartDefault)]
#[sqlx(default)]
pub struct LessonWithAuthorTopic {
    #[sqlx(flatten)]
    pub lesson: LessonEntity,
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
    //favourite
    pub is_favourite: bool,
    //progress
    pub user_id: Option<i64>,
    pub started_at: Option<OffsetDateTime>,
    pub last_position_ms: Option<i64>,
    pub status: Option<UserProgressStatus>,
    pub completed_at: Option<OffsetDateTime>
}

#[derive(Debug, FromRow)]
pub struct LessonProgressEntity {
    pub user_id: i64,
    pub author_id: i64,
    pub topic_id: Option<i64>,
    pub lesson_id: i64,
    pub started_at: OffsetDateTime,
    pub last_position_ms: i64,
    pub status: UserProgressStatus,
    pub completed_at: Option<OffsetDateTime>
}