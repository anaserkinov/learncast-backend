use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use fluent_templates::LanguageIdentifier;
use serde_json::Value;
use thiserror::Error;
use crate::error::AppError;
use crate::module::common::base::BaseResponse;
use crate::string_keys::strings;
use crate::utils::t;

#[derive(Error, Debug, Clone)]
pub enum SnipError {
    #[error("SnipNotOwnedUpdate")]
    SnipNotOwnedUpdate(LanguageIdentifier),
    #[error("SnipNotOwnedDelete")]
    SnipNotOwnedDelete(LanguageIdentifier)
}

impl From<SnipError> for AppError {
    fn from(value: SnipError) -> Self { AppError::Snip(value) }
}

impl IntoResponse for SnipError {
    fn into_response(self) -> Response {
        let (status, code, lang, message_key, data_payload) = match self {
            SnipError::SnipNotOwnedUpdate(lang) => (StatusCode::FORBIDDEN, 105001, lang, strings::SNIP_NOT_OWNED_UPDATE, None),
            SnipError::SnipNotOwnedDelete(lang) => (StatusCode::FORBIDDEN, 105002, lang, strings::SNIP_NOT_OWNED_DELETE, None)
        };

        let body = axum::Json(
            BaseResponse::<Value>::error(
                code,
                &t(&lang, message_key),
                data_payload
            )
        );

        (status, body).into_response()
    }
}