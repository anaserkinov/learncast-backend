use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct CommonTopicResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub lesson_count: i64,
    pub total_duration: i64
}

#[derive(Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QuerySort {
    SnipCount,
    CreatedAt
}