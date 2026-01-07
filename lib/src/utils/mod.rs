pub mod telegram;
pub mod jwt;
pub mod cursor;

use crate::config::AppConfig;
use fluent_templates::{static_loader, LanguageIdentifier, Loader};
use std::sync::LazyLock;

static_loader! {
    pub static LOCALES = {
        locales: "locales",
        fallback_language: "en",
    };
}

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {AppConfig::load()});

pub fn t(lang_id: &LanguageIdentifier, key: &str) -> String {
    LOCALES.lookup(lang_id, key)
}

pub mod extractors {
    use crate::error::{AppError};
    use axum::extract::{FromRequestParts, Path, Query, Request};
    use axum::http::request::Parts;
    use axum::{
        extract::FromRequest,
        Json
    };
    use serde::de::DeserializeOwned;
    use validator::Validate;
    use crate::extractor::accept_language::AcceptLanguage;

    pub struct ValidatedJson<T>(pub T);

    impl<S, T> FromRequest<S> for ValidatedJson<T>
    where
        T: DeserializeOwned + Validate,
        S: Send + Sync,
    {
        type Rejection = AppError;

        async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
            let lang_header = req.headers().get("accept-language").cloned();

            let Json::<T>(value) = Json::from_request(req, &state).await
                .map_err(|err|
                    AppError::BadRequest{
                        lang: AcceptLanguage::from(lang_header.as_ref()).0,
                        message: err.body_text()
                    }
                )?;
            
            if let Err(err) = value.validate() {
                return Err(AppError::BadRequest{
                    lang: AcceptLanguage::from(lang_header.as_ref()).0,
                    message: err.0.iter().next().unwrap().0.to_string()
                });
            };

            Ok(ValidatedJson(value))
        }
    }

    pub struct ValidatedQuery<T>(pub T);

    impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
    where
        T: DeserializeOwned + Send,
        S: Send + Sync,
    {
        type Rejection = AppError;

        async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
            let value = match Query::<T>::from_request_parts(parts, _state).await {
                Ok(Query(v)) => v,
                Err(err) => {
                    return Err(AppError::BadRequest {
                        lang: AcceptLanguage::from_request_parts(parts, _state).await?.0,
                        message: err.body_text(),
                    });
                }
            };
            Ok(ValidatedQuery(value))
        }
    }

    pub struct ValidatedPath<T>(pub T);

    impl<T, S> FromRequestParts<S> for ValidatedPath<T>
    where
        T: DeserializeOwned + Send,
        S: Send + Sync,
    {
        type Rejection = AppError;

        async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
            let value = match Path::from_request_parts(parts, _state).await {
                Ok(Path(v)) => v,
                Err(err) => {
                    return Err(AppError::BadRequest {
                        lang: AcceptLanguage::from_request_parts(parts, _state).await?.0,
                        message: err.body_text(),
                    });
                }
            };
            Ok(ValidatedPath(value))
        }
    }
}
