use axum::Extension;
use crate::state::AppState;
use axum::extract::State;
use crate::error::AppError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::module::common::base::{BaseResponse, DeletedParams, DeletedResponse, IdParam};
use crate::module::user::lesson::dto::{LessonPaginationParams, ListenSessionCreateRequest, ListenSessionCreateResponse};
use crate::module::common::lesson::service;
use crate::module::common::paging::{CursorPagingResponse, PagingResponse};
use crate::module::user::lesson::dto::{LessonProgressResponse, LessonProgressUpdateRequest, LessonResponse};
use crate::module::user::lesson::mapper;
use crate::utils::extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};
use crate::utils::jwt::Claims;

#[utoipa::path(
    get,
    path = "/v1/user/lesson",
    security(("bearerAuth" = [])),
    params(LessonPaginationParams),
    responses((status = 200, body = CursorPagingResponse<LessonResponse>)),
    tag = "Lesson"
)]
pub async fn page_lesson(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<LessonPaginationParams>,
    Extension(claims): Extension<Claims>
) -> Result<BaseResponse<CursorPagingResponse<LessonResponse>>, AppError> {
    let lessons = service::page_cursor(
        &state.db,
        params.limit,
        params.cursor,
        params.author_id,
        params.topic_id,
        params.search,
        Some(claims.sub),
        params.status,
        params.favourite,
        params.sort,
        params.order
    ).await?;

    Ok(
        BaseResponse::success(
            CursorPagingResponse::new(
                lessons.0.into_iter().map(mapper::to_response).collect(),
                lessons.1
            )
        )
    )
}

#[utoipa::path(
    get,
    path = "/v1/user/lesson/deleted",
    security(("bearerAuth" = [])),
    params(DeletedParams),
    responses((status = 200, body = Vec<DeletedResponse>)),
    tag = "Lesson"
)]
pub async fn deleted_lessons(
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
    post,
    path = "/v1/user/lesson/{id}/favourite",
    security(("bearerAuth" = [])),
    params(IdParam),
    tag = "Lesson"
)]
pub async fn set_favourite(
    State(state): State<AppState>,
    ValidatedPath(id): ValidatedPath<i64>,
    Extension(claims): Extension<Claims>
) -> Result<BaseResponse<()>, AppError> {
    service::set_favourite(
        &state.db,
        claims.sub,
        id
    ).await?;

    Ok(BaseResponse::success(()))
}

#[utoipa::path(
    delete,
    path = "/v1/user/lesson/{id}/favourite",
    security(("bearerAuth" = [])),
    params(IdParam),
    tag = "Lesson"
)]
pub async fn remove_favourite(
    State(state): State<AppState>,
    ValidatedPath(id): ValidatedPath<i64>,
    Extension(claims): Extension<Claims>
) -> Result<BaseResponse<()>, AppError> {
    service::remove_favourite(
        &state.db,
        claims.sub,
        id
    ).await?;

    Ok(BaseResponse::success(()))
}


#[utoipa::path(
    post,
    path = "/v1/user/lesson/{id}/listen",
    security(("bearerAuth" = [])),
    params(IdParam),
    request_body = ListenSessionCreateRequest,
    responses((status = 200, body = ListenSessionCreateResponse)),
    tag = "Lesson"
)]
pub async fn increase_listen_count(
    State(state): State<AppState>,
    ValidatedPath(id): ValidatedPath<i64>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(body): ValidatedJson<ListenSessionCreateRequest>
) -> Result<BaseResponse<ListenSessionCreateResponse>, AppError> {
    let listen_count = service::increase_listen_count(
        &state.db,
        body.session_id,
        claims.sub,
        id,
        body.created_at
    ).await?;

    Ok(BaseResponse::success(
        ListenSessionCreateResponse { listen_count }
    ))
}

#[utoipa::path(
    patch,
    path = "/v1/user/lesson/{id}/progress",
    security(("bearerAuth" = [])),
    params(IdParam),
    request_body = LessonProgressUpdateRequest,
    responses((status = 200, body = PagingResponse<LessonProgressResponse>)),
    tag = "Lesson"
)]
pub async fn update_lesson_progress(
    State(state): State<AppState>,
    ValidatedPath(id): ValidatedPath<i64>,
    Extension(claims): Extension<Claims>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedJson(body): ValidatedJson<LessonProgressUpdateRequest>
) -> Result<BaseResponse<LessonProgressResponse>, AppError> {
    let entity = service::update_progress(
        &state.db,
        claims.sub,
        id,
        body.started_at,
        body.last_position_ms,
        body.status,
        body.completed_at,
        lang
    ).await?;

    Ok(
        BaseResponse::success(
            mapper::progress_to_response(entity)
        )
    )
}