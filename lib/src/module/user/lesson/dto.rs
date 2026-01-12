use crate::module::common::enums::{UserProgressStatus};
use crate::module::common::lesson::dto::{CommonLessonResponse, QuerySort};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
use crate::module::common::paging::QueryOrder;

#[derive(Debug, Serialize, ToSchema)]
pub struct LessonResponse {
    #[serde(flatten)]
    pub lesson: CommonLessonResponse,
    pub is_favourite: bool,
    pub lesson_progress: Option<LessonProgressResponse>
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LessonProgressResponse {
    pub user_id: i64,
    pub author_id: i64,
    pub topic_id: Option<i64>,
    pub lesson_id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
    pub last_position_ms: i64,
    pub status: UserProgressStatus,
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LessonProgressUpdateRequest {
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
    pub last_position_ms: i64,
    pub status: Option<UserProgressStatus>,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub completed_at: Option<OffsetDateTime>
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListenSessionCreateRequest {
    pub session_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime
}

#[derive(Debug, Serialize, Validate, ToSchema)]
pub struct ListenSessionCreateResponse {
    pub listen_count: i64
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LessonPaginationParams {
    #[param(example = json!(20))]
    pub limit: u32,
    pub search: Option<String>,
    pub order: Option<QueryOrder>,

    pub cursor: Option<String>,

    pub status: Option<UserProgressStatus>,
    pub sort: Option<QuerySort>,
    pub author_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub favourite: Option<bool>
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LessonCursor {
    pub id: i64,
    pub snip_count: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}