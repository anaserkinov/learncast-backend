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
pub enum AuthorError {
    #[error("AuthorHasLesson")]
    AuthorHasLesson(LanguageIdentifier)
}

impl From<AuthorError> for AppError {
    fn from(value: AuthorError) -> Self { AppError::Author(value) }
}

impl IntoResponse for AuthorError {
    fn into_response(self) -> Response {
        let (status, code, lang, message_key, data_payload) = match self {
            AuthorError::AuthorHasLesson(lang) => (StatusCode::CONFLICT, 102001, lang, strings::AUTHOR_HAS_LESSON, None)
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