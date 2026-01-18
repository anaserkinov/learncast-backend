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
pub enum TopicError {
    #[error("AuthorHasLesson")]
    TopicHasLesson(LanguageIdentifier)
}

impl From<TopicError> for AppError {
    fn from(value: TopicError) -> Self { AppError::Topic(value) }
}

impl IntoResponse for TopicError {
    fn into_response(self) -> Response {
        let (status, code, lang, message_key, data_payload) = match self {
            TopicError::TopicHasLesson(lang) => (StatusCode::CONFLICT, 103001, lang, strings::TOPIC_HAS_LESSON, None)
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