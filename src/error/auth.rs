use axum::http::StatusCode;
use crate::module::common::base::BaseResponse;
use crate::utils::t;
use axum::response::{IntoResponse, Response};
use fluent_templates::LanguageIdentifier;
use serde_json::Value;
use thiserror::Error;
use crate::error::AppError;
use crate::string_keys::strings;

#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("Unauthorized")]
    Unauthorized(LanguageIdentifier),
    #[error("InvalidCredentials")]
    InvalidCredentials(LanguageIdentifier)
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self { AppError::Auth(value) }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, lang, message_key, data_payload) = match self {
            AuthError::Unauthorized(lang) => (StatusCode::UNAUTHORIZED, 101001, lang, strings::UNAUTHORIZED_USER, None),
            AuthError::InvalidCredentials(lang) => (StatusCode::UNAUTHORIZED, 101002, lang, strings::INVALID_CREDENTIALS, None)
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