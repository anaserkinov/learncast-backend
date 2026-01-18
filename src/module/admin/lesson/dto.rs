use crate::module::common::lesson::dto::{CommonLessonResponse, QuerySort};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
use crate::module::common::enums::UserProgressStatus;
use crate::module::common::paging::QueryOrder;

#[derive(Debug, Serialize, ToSchema)]
pub struct LessonResponse {
    #[serde(flatten)]
    pub lesson: CommonLessonResponse,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LessonCURequest {
    pub author_id: i64,
    pub topic_id: Option<i64>,
    #[validate(length(min = 3))]
    pub title: String,
    pub description: Option<String>,
    pub cover_image_path: Option<String>,
    pub audio_path: String
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LessonPaginationParams {
    #[param(example = json!(1))]
    pub page: Option<u32>,
    #[param(example = json!(20))]
    pub limit: Option<u32>,
    pub search: Option<String>,
    pub order: Option<QueryOrder>,

    pub status: Option<UserProgressStatus>,
    pub sort: Option<QuerySort>,
    pub author_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub favourite: Option<bool>
}