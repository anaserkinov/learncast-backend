use axum::{
    middleware::Next,
    response::Response,
};
use axum::extract::{Request};
use axum::http::header;
use axum_extra::extract::CookieJar;
use crate::error::AppError;
use crate::error::auth::AuthError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::utils::{jwt, CONFIG};

pub async fn common_auth_middleware(
    AcceptLanguage(lang): AcceptLanguage,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {

    let token = if let Some(header_value) = req.headers().get(header::AUTHORIZATION) {
        let header = header_value.to_str()
            .map_err(|_| AuthError::Unauthorized(lang.clone()))?;
        &header[7..]
    } else {
        let jar = CookieJar::from_headers(req.headers());
        &jar.get("access_token")
            .map(|c| c.value().to_string())
            .ok_or(AuthError::Unauthorized(lang.clone()))?
    };

    let claims = jwt::validate_access_token(token)
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;

    req.extensions_mut().insert(AcceptLanguage(lang));
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

pub async fn origin_middleware(
    AcceptLanguage(lang): AcceptLanguage,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    
    if !cfg!(debug_assertions) {
        let Some(header_value) = req.headers().get(header::ORIGIN) else {
            return Err(AuthError::Unauthorized(lang).into());
        };

        let header = header_value.to_str()
            .map_err(|_| AuthError::Unauthorized(lang.clone()))?;

        if !header.starts_with(CONFIG.client_origin.as_str()) {
            return Err(AuthError::Unauthorized(lang).into());
        }
    }

    Ok(next.run(req).await)
}


pub async fn admin_auth_middleware(
    AcceptLanguage(lang): AcceptLanguage,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {

    let jar = CookieJar::from_headers(req.headers());

    let token = jar.get("access_token")
        .map(|c| c.value().to_string())
        .ok_or(AuthError::Unauthorized(lang.clone()))?;

    let claims = jwt::validate_access_token(&token)
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;

    if claims.role != "admin" {
        return Err(AuthError::InvalidCredentials(lang).into());
    }

    req.extensions_mut().insert(AcceptLanguage(lang));
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

pub async fn user_auth_middleware(
    AcceptLanguage(lang): AcceptLanguage,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {

    let Some(header_value) = req.headers().get(header::AUTHORIZATION) else {
        return Err(AuthError::Unauthorized(lang).into());
    };

    let header = header_value.to_str()
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;

    let token = &header[7..];
    let claims = jwt::validate_access_token(token)
        .map_err(|_| AuthError::Unauthorized(lang.clone()))?;

    if claims.role != "user" {
        return Err(AuthError::InvalidCredentials(lang).into());
    }

    req.extensions_mut().insert(AcceptLanguage(lang));
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
