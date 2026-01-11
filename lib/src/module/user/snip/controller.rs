use crate::error::AppError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::module::common::base::{BaseResponse, ClientSnipIdParam, DeletedParams, DeletedResponse, LessonIdParam};
use crate::module::common::paging::CursorPagingResponse;
use crate::module::user::snip::dto::{SnipCURequest, SnipCountResponse, SnipPaginationParams, SnipResponse};
use crate::module::user::snip::mapper;
use crate::module::user::snip::service;
use crate::state::AppState;
use crate::utils::extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};
use crate::utils::jwt::Claims;
use axum::extract::State;
use axum::Extension;

#[utoipa::path(
    post,
    path = "/v1/user/lesson/{lesson_id}/snip",
    security(("bearerAuth" = [])),
    params(LessonIdParam),
    request_body = SnipCURequest,
    responses((status = 200, body = SnipResponse)),
    tag = "Snip"
)]
pub async fn create_snip(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(lesson_id): ValidatedPath<i64>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(body): ValidatedJson<SnipCURequest>
) -> Result<BaseResponse<SnipResponse>, AppError> {

    let snip = service::create(
        &state.db,
        body.client_snip_id,
        lesson_id,
        claims.sub,
        body.start_ms,
        body.end_ms,
        body.note_text,
        body.created_at,
        lang
    ).await?;

    let user_snip_count = service::count(
        &state.db,
        claims.sub,
        lesson_id
    ).await?;

    Ok(
        BaseResponse::success(
            mapper::to_response(snip, Some(user_snip_count))
        )
    )
}

#[utoipa::path(
    put,
    path = "/v1/user/lesson/snip/{client_snip_id}",
    security(("bearerAuth" = [])),
    params(ClientSnipIdParam),
    request_body = SnipCURequest,
    responses((status = 200, body = SnipResponse)),
    tag = "Snip"
)]
pub async fn update_snip(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(client_snip_id): ValidatedPath<String>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(body): ValidatedJson<SnipCURequest>,
) -> Result<BaseResponse<SnipResponse>, AppError> {

    let snip = service::update(
        &state.db,
        client_snip_id,
        claims.sub,
        body.start_ms,
        body.end_ms,
        body.note_text,
        lang
    ).await?;

    Ok(
        BaseResponse::success(
            mapper::to_response(snip, None)
        )
    )
}

#[utoipa::path(
    delete,
    path = "/v1/user/lesson/snip/{client_snip_id}",
    security(("bearerAuth" = [])),
    params(ClientSnipIdParam),
    tag = "Snip"
)]
pub async fn delete_snip(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(client_snip_id): ValidatedPath<String>,
    Extension(claims): Extension<Claims>
) -> Result<BaseResponse<()>, AppError> {

    service::delete(
        &state.db,
        client_snip_id,
        claims.sub,
        lang
    ).await?;

    Ok(
        BaseResponse::success(())
    )
}

#[utoipa::path(
    get,
    path = "/v1/user/lesson/snip",
    security(("bearerAuth" = [])),
    params(SnipPaginationParams),
    responses((status = 200, body = CursorPagingResponse<SnipResponse>)),
    tag = "Snip"
)]
pub async fn page_snip(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<SnipPaginationParams>,
    Extension(claims): Extension<Claims>
) -> Result<BaseResponse<CursorPagingResponse<SnipResponse>>, AppError> {

    let topics = service::page(
        &state.db,
        params.limit,
        params.cursor,
        claims.sub,
        params.lesson_id,
        params.search,
        params.sort,
        params.order
    ).await?;

    Ok(
        BaseResponse::success(
            CursorPagingResponse::new(
                topics.0.into_iter()
                    .map(|t| mapper::to_response(t, None))
                    .collect(),
                topics.1
            )
        )
    )
}

#[utoipa::path(
    get,
    path = "/v1/user/lesson/snip/deleted",
    security(("bearerAuth" = [])),
    params(DeletedParams),
    responses((status = 200, body = Vec<DeletedResponse>)),
    tag = "Snip"
)]
pub async fn deleted_snips(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<DeletedParams>
) -> Result<BaseResponse<Vec<DeletedResponse>>, AppError> {
    Ok(
        BaseResponse::success(
            service::deleted(&state.db, params.since).await?
                .iter().map(|pair| {
                DeletedResponse{ id: pair.0, deleted_at: pair.1 }
            }).collect()
        )
    )
}

#[utoipa::path(
    get,
    path = "/v1/user/lesson/{lesson_id}/snip/count",
    security(("bearerAuth" = [])),
    params(LessonIdParam),
    responses((status = 200, body = SnipCountResponse)),
    tag = "Snip"
)]
pub async fn count_snip(
    State(state): State<AppState>,
    ValidatedPath(lesson_id): ValidatedPath<i64>,
    Extension(claims): Extension<Claims>
) -> Result<BaseResponse<SnipCountResponse>, AppError> {

    let count = service::count(
        &state.db,
        claims.sub,
        lesson_id
    ).await?;

    Ok(
        BaseResponse::success(
            SnipCountResponse{ count }
        )
    )
}