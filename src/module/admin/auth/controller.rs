use crate::error::auth::AuthError;
use crate::error::AppError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::module::common::auth::dto::{SignInRequest, UserResponse};
use crate::module::common::auth::{mapper, service};
use crate::module::common::base::BaseResponse;
use crate::state::AppState;
use crate::utils::extractors::ValidatedJson;
use crate::utils::jwt::Claims;
use crate::utils::{jwt, CONFIG};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use axum_extra::TypedHeader;
use headers::UserAgent;
use time::Duration;

#[utoipa::path(
    get,
    path = "/v1/admin/me",
    security(("cookieAuth" = [])),
    responses(
        (status = 200, body = UserResponse)
    ),
    tag = "Auth"
)]
pub async fn get_me(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    Extension(claims): Extension<Claims>,
) -> Result<BaseResponse<UserResponse>, AppError> {
    let user_id = claims.sub;
    let user = service::get_me(
        &state.db,
        user_id,
        lang,
    ).await?;

    Ok(
        BaseResponse::success(
            mapper::to_response(user)
        )
    )
}

#[utoipa::path(
    post,
    path = "/v1/admin/auth/signin",
    request_body = SignInRequest,
    responses(
        (
            status = 200,
            body = UserResponse,
            description = "Tokens are returned via Set-Cookie headers.",
            headers(
                ("Set-Cookie" = String, description = "access_token and refresh_token cookies")
            )
        )
    ),
    tag = "Auth"
)]
pub async fn signin(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ValidatedJson(body): ValidatedJson<SignInRequest>,
) -> Result<Response, AppError> {
    let result = if let Some(telegram_data) = body.telegram_data {
        service::signin_with_telegram(
            &state.db,
            user_agent.to_string(),
            "admin".into(),
            telegram_data,
            lang,
        )
            .await
    } else if let Some(google_data) = body.google_data {
        service::signin_with_google(
            &state.db,
            user_agent.to_string(),
            "admin".into(),
            google_data,
            lang,
        )
            .await
    } else {
        return Err(AuthError::InvalidCredentials(lang).into());
    }?;

    Ok(
        (
            build_cookie(
                result.1,
                result.2,
            ),
            BaseResponse::success(mapper::to_response(result.0))
        ).into_response()
    )
}

#[utoipa::path(
    post,
    path = "/v1/admin/auth/refresh-token",
    responses(
        (
            status = 200,
            description = "Access and refresh tokens rotated successfully.
            Tokens are returned via Set-Cookie headers.",
            headers(
                ("Set-Cookie" = String, description = "access_token and refresh_token cookies")
            )
        )
    ),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    AcceptLanguage(lang): AcceptLanguage,
    request: Request,
) -> Result<Response, AppError> {
    let jar = CookieJar::from_headers(request.headers());

    let refresh_token = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AuthError::Unauthorized(lang.clone()))?;

    let result = service::refresh_tokens(
        &state.db,
        refresh_token,
        user_agent.to_string(),
        "admin".into(),
        lang,
    )
        .await?;

    Ok(
        (
            build_cookie(
                result.0,
                result.1,
            ),
            StatusCode::OK,
        ).into_response()
    )
}

fn get_main_domain(url: &str) -> String {
    let without_scheme = if let Some(pos) = url.find("://") {
        &url[pos + 3..]
    } else {
        url
    };
    let host = without_scheme;

    let parts: Vec<&str> = host.split('.').collect();

    format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1])
}

fn build_cookie(
    refresh_token: String,
    access_token: String,
) -> CookieJar {
    let domain = get_main_domain(CONFIG.client_origin.as_str());

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .path(format!("{}/v1/admin/auth/refresh-token", CONFIG.base_path))
        .domain(domain.clone())
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::days(7))
        .build();

    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .domain(domain)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::minutes(15))
        .build();

    let jar = CookieJar::new().add(refresh_cookie).add(access_cookie);
    jar
}

#[utoipa::path(
    post,
    path = "/v1/admin/auth/logout",
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    request: Request,
) -> Result<BaseResponse<()>, AppError> {
    let jar = CookieJar::from_headers(request.headers());

    let access_token = jar
        .get("access_token")
        .map(|c| c.value())
        .unwrap_or("");

    if let Ok(claims) = jwt::validate_access_token(access_token) {
        service::logout(&state.db, claims.sub).await?;
    }

    Ok(BaseResponse::empty())
}
