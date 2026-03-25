use crate::error::AppError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::module::common::auth::dto::{Credentials, LoginRequest, LoginResponse, RefreshTokenRequest};
use crate::module::common::auth::{mapper, service};
use crate::module::common::base::BaseResponse;
use crate::state::AppState;
use crate::utils::jwt;
use axum::extract::State;
use axum::Json;
use axum_extra::TypedHeader;
use headers::authorization::Bearer;
use headers::{Authorization, UserAgent};

#[utoipa::path(
    post,
    path = "/v1/user/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = LoginResponse)
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Json(body): Json<LoginRequest>,
) -> Result<BaseResponse<LoginResponse>, AppError> {
    let result = match body {
        LoginRequest::Telegram { data } => {
            service::login_with_telegram(
                &state.db,
                &state.jwks_cache,
                user_agent.to_string(),
                "user".into(),
                data,
                lang,
            ).await
        },
        LoginRequest::Google { data } =>  {
            service::login_with_google(
                &state.db,
                user_agent.to_string(),
                "user".into(),
                data,
                lang,
            ).await
        }
    }?;

    Ok(BaseResponse::success(LoginResponse {
        user: mapper::to_response(result.0),
        credentials: Credentials{
            refresh_token: result.1,
            access_token: result.2
        }
    }))
}

#[utoipa::path(
    post,
    path = "/v1/user/auth/refresh-token",
    security(("bearerAuth" = [])),
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, body = Credentials)
    ),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    AcceptLanguage(lang): AcceptLanguage,
    Json(body): Json<RefreshTokenRequest>,
) -> Result<BaseResponse<Credentials>, AppError> {
    let result = service::refresh_tokens(
        &state.db,
        body.refresh_token,
        user_agent.to_string(),
        "user".into(),
        lang
    ).await?;

    Ok(BaseResponse::success(
        Credentials{
            refresh_token: result.0,
            access_token: result.1
        }
    ))
}

#[utoipa::path(
    post,
    path = "/v1/user/auth/logout",
    security(("bearerAuth" = [])),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>
) -> Result<BaseResponse<()>, AppError> {
    if let Ok(claims) = jwt::validate_access_token(auth.0.token()) {
        service::logout(
            &state.db,
            claims.sub
        ).await?;
    }

    Ok(BaseResponse::empty())
}
