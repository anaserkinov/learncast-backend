pub mod auth;
pub mod author;
pub mod lesson;
pub mod topic;
pub mod snip;

use crate::error::auth::AuthError;
use crate::error::author::AuthorError;
use crate::error::lesson::LessonError;
use crate::error::topic::TopicError;
use crate::module::common::base::BaseResponse;
use crate::string_keys::strings;
use crate::utils::t;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use fluent_templates::LanguageIdentifier;
use serde_json::Value;
use sqlx::Error;
use std::fmt::Debug;
use thiserror::Error;
use crate::error::snip::SnipError;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("NotFound")]
    NotFound(LanguageIdentifier),
    #[error("BadRequest")]
    BadRequest {
        lang: LanguageIdentifier,
        message: String,
    },
    #[error("Internal")]
    Internal(LanguageIdentifier),
    #[error("UnsupportedFileType")]
    UnsupportedFileType(LanguageIdentifier),
    #[error("FileTooLarge")]
    FileTooLarge(LanguageIdentifier),

    #[error(transparent)]
    Auth(AuthError),
    #[error(transparent)]
    Author(AuthorError),
    #[error(transparent)]
    Topic(TopicError),
    #[error(transparent)]
    Lesson(LessonError),
    #[error(transparent)]
    Snip(SnipError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, data_payload) = match self {
            AppError::NotFound(lang) => (
                StatusCode::NOT_FOUND,
                100001,
                t(&lang, strings::NOT_FOUND),
                None,
            ),
            AppError::BadRequest { lang, message } => (
                StatusCode::BAD_REQUEST,
                100002,
                message,
                None
            ),
            AppError::Internal(lang) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                100003,
                t(&lang, strings::INTERNAL_ERROR),
                None,
            ),
            AppError::UnsupportedFileType(lang) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                100004,
                t(&lang, strings::UNSUPPORTED_FILE_TYPE),
                None,
            ),
            AppError::FileTooLarge(lang) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                100005,
                t(&lang, strings::FILE_TOO_LARGE),
                None,
            ),

            AppError::Auth(err) => return err.into_response(),
            AppError::Author(err) => return err.into_response(),
            AppError::Topic(err) => return err.into_response(),
            AppError::Lesson(err) => return err.into_response(),
            AppError::Snip(err) => return err.into_response(),
        };

        let body = axum::Json(BaseResponse::<Value>::error(
            code,
            &message,
            data_payload,
        ));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: Error) -> Self {
        AppError::Internal("en-US".parse().unwrap())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        eprintln!("Unhandled anyhow error: {:?}", error);

        for cause in error.chain() {
            if let Some(app_err) = cause.downcast_ref::<AppError>() {
                return app_err.clone();
            }
        }

        AppError::Internal("en-US".parse().unwrap())
    }
}
