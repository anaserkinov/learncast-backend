use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use axum::http::{header, HeaderValue};
use fluent_templates::LanguageIdentifier;
use crate::error::AppError;

#[derive(Clone)]
pub struct AcceptLanguage(pub LanguageIdentifier);

impl AcceptLanguage {
    pub fn from(header: Option<&HeaderValue>) -> Self {
        let lang = header
            .and_then(|v| v.to_str().ok())
            .unwrap_or("en")
            .split(',')
            .next()
            .unwrap_or("en")
            .trim();

        AcceptLanguage(
            lang.parse::<LanguageIdentifier>()
                .unwrap_or_else(|_| "en".parse().unwrap())
        )
    }
}

impl<S> FromRequestParts<S> for AcceptLanguage
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(
            AcceptLanguage::from(
                parts.headers.get(header::ACCEPT_LANGUAGE)
            )
        )
    }
}


