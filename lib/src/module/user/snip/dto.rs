use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
use crate::module::common::enums::{UserProgressStatus};
use crate::module::common::lesson::dto::{CommonLessonResponse};
use crate::module::common::paging::QueryOrder;

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct SnipCURequest{
    pub client_snip_id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub note_text: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime
}

#[derive(Serialize, Debug, ToSchema)]
pub struct SnipResponse {
    pub id: i64,
    pub client_snip_id: String,
    pub user_id: i64,
    pub lesson: CommonLessonResponse,
    pub start_ms: i64,
    pub end_ms: i64,
    pub note_text: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub user_snip_count: Option<i64>
}

#[derive(Serialize, Debug, ToSchema)]
pub struct SnipCountResponse {
    pub count: i64
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SnipPaginationParams {
    #[param(example = json!(20))]
    pub limit: u32,
    pub search: Option<String>,
    pub order: Option<QueryOrder>,

    pub cursor: Option<String>,
    
    pub status: Option<UserProgressStatus>,
    pub sort: Option<QuerySort>,
    pub lesson_id: Option<i64>
}

#[derive(Serialize, Deserialize)]
pub struct SnipCursor{
    pub id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuerySort {
    CreatedAt
}
