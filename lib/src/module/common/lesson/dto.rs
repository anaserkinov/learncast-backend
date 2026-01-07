use crate::module::common::author::dto::CommonAuthorResponse;
use crate::module::common::base::FileResponse;
use crate::module::common::topic::dto::CommonTopicResponse;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct CommonLessonResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    pub author: CommonAuthorResponse,
    pub topic: Option<CommonTopicResponse>,
    pub audio: FileResponse,
    pub listen_count: i64,
    pub snip_count: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime
}

#[derive(Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuerySort {
    SnipCount,
    CreatedAt
}