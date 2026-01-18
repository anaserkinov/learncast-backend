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
pub enum LessonError {
    #[error("LessonDeleteTooManyListens")]
    LessonDeleteTooManyListens(LanguageIdentifier)
}

impl From<LessonError> for AppError {
    fn from(value: LessonError) -> Self { AppError::Lesson(value) }
}

impl IntoResponse for LessonError {
    fn into_response(self) -> Response {
        let (status, code, lang, message_key, data_payload) = match self {
            LessonError::LessonDeleteTooManyListens(lang) => (StatusCode::CONFLICT, 104001, lang, strings::LESSON_DELETE_TOO_MANY_LISTENS, None)
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