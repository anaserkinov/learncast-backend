use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use validator::{Validate};
use crate::module::common::author::dto::CommonAuthorResponse;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorResponse {
    #[serde(flatten)]
    pub author: CommonAuthorResponse
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AuthorCURequest {
    #[validate(length(min = 3))]
    pub name: String,
    pub avatar_path: Option<String>
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    #[param(example = json!(1))]
    pub page: Option<u32>,
    #[param(example = json!(20))]
    pub limit: Option<u32>,
    pub search: Option<String>
}