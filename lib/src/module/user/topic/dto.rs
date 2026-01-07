use crate::module::common::author::dto::CommonAuthorResponse;
use crate::module::common::enums::UserProgressStatus;
use crate::module::common::paging::QueryOrder;
use crate::module::common::topic::dto::{CommonTopicResponse, QuerySort};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct TopicResponse {
    pub id: i64,
    pub topic: CommonTopicResponse,
    pub author: CommonAuthorResponse,
    pub completed_lesson_count: i64
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TopicPaginationParams {
    pub limit: u32,
    pub search: Option<String>,
    pub order: Option<QueryOrder>,

    pub cursor: Option<String>,
    
    pub author_id: Option<i64>,
    pub status: Option<UserProgressStatus>,
    pub sort: Option<QuerySort>
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TopicCursor {
    pub id: i64,
    pub snip_count: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}


