use axum::Extension;
use axum::extract::State;
use crate::error::AppError;
use crate::module::common::base::{BaseResponse, DeletedParams, DeletedResponse};
use crate::module::common::paging::{CursorPagingResponse};
use crate::module::common::topic::service;
use crate::module::user::topic::dto::{TopicPaginationParams, TopicResponse};
use crate::module::user::topic::mapper;
use crate::state::AppState;
use crate::utils::extractors::ValidatedQuery;
use crate::utils::jwt::Claims;

#[utoipa::path(
    get,
    path = "/v1/user/topic",
    security(("bearerAuth" = [])),
    params(TopicPaginationParams),
    responses((status = 200, body = CursorPagingResponse<TopicResponse>)),
    tag = "Topic"
)]
pub async fn page_topic(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<TopicPaginationParams>,
    Extension(claims): Extension<Claims>
) -> Result<BaseResponse<CursorPagingResponse<TopicResponse>>, AppError> {
    let topics = service::page_with_author(
        &state.db,
        params.limit,
        params.cursor,
        params.search,
        Some(claims.sub),
        params.author_id,
        params.status,
        params.sort,
        params.order
    ).await?;
    
    Ok(
        BaseResponse::success(
            CursorPagingResponse::new(
                topics.0.into_iter().map(mapper::to_response).collect(),
                topics.1
            )
        )
    )
}

#[utoipa::path(
    get,
    path = "/v1/user/topic/deleted",
    security(("bearerAuth" = [])),
    params(DeletedParams),
    responses((status = 200, body = Vec<DeletedResponse>)),
    tag = "Topic"
)]
pub async fn deleted_topics(
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