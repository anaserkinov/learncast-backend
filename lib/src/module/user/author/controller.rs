use crate::error::AppError;
use crate::module::common::author::service;
use crate::module::common::base::{BaseResponse, DeletedParams, DeletedResponse};
use crate::module::common::paging::CursorPagingResponse;
use crate::module::user::author::dto::{AuthorResponse, AuthorPaginationParams};
use crate::module::user::author::mapper;
use crate::state::AppState;
use crate::utils::extractors::ValidatedQuery;
use axum::extract::State;

#[utoipa::path(
    get,
    path = "/v1/user/author",
    security(("bearerAuth" = [])),
    params(AuthorPaginationParams),
    responses((status = 200, body = CursorPagingResponse<AuthorResponse>)),
    tag = "Author"
)]
pub async fn page_author(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<AuthorPaginationParams>
) -> Result<BaseResponse<CursorPagingResponse<AuthorResponse>>, AppError> {
    let topics = service::page_cursor(
        &state.db,
        params.limit,
        params.cursor,
        params.search
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
    path = "/v1/user/author/deleted",
    security(("bearerAuth" = [])),
    params(DeletedParams),
    responses((status = 200, body = Vec<DeletedResponse>)),
    tag = "Author"
)]
pub async fn deleted_authors(
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