use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use validator::{Validate};
use crate::module::common::enums::UserProgressStatus;
use crate::module::common::paging::QueryOrder;
use crate::module::common::topic::dto::{CommonTopicResponse, QuerySort};

#[derive(Debug, Serialize, ToSchema)]
pub struct TopicResponse {
    #[serde(flatten)]
    pub topic: CommonTopicResponse
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct TopicCURequest {
    #[validate(length(min = 3))]
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    #[param(example = json!(1))]
    pub page: u32,
    #[param(example = json!(20))]
    pub limit: u32,
    pub search: Option<String>,
    pub order: Option<QueryOrder>,

    pub author_id: Option<i64>,
    pub status: Option<UserProgressStatus>,
    pub sort: Option<QuerySort>
}