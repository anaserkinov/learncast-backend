use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use crate::module::common::author::dto::CommonAuthorResponse;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorResponse {
    #[serde(flatten)]
    pub author: CommonAuthorResponse
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AuthorPaginationParams {
    pub limit: u32,
    pub search: Option<String>,
    pub cursor: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct AuthorCursor {
    pub id: i64,
    pub lesson_count: i64
}