use crate::state::AppState;
use axum::extract::{State};
use crate::error::AppError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::module::admin::author::dto::{AuthorCURequest, AuthorResponse};
use crate::module::admin::author::{mapper};
use crate::module::common::base::{BaseResponse, IdParam};
use crate::module::admin::author::dto::PaginationParams;
use crate::module::common::paging::PagingResponse;
use crate::module::common::author::service;
use crate::utils::extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};

#[utoipa::path(
    post,
    path = "/v1/admin/author",
    security(("cookieAuth" = [])),
    request_body = AuthorCURequest,
    responses((status = 200, body = AuthorResponse)),
    tag = "Author"
)]
pub async fn create_author(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<AuthorCURequest>,
) -> Result<BaseResponse<AuthorResponse>, AppError> {
    
    let author = service::create(
        &state.db,
        body.name,
        body.avatar_path
    ).await?;

    Ok(
        BaseResponse::success(
            mapper::to_response(author)
        )
    )
}

#[utoipa::path(
    put,
    path = "/v1/admin/author/{id}",
    security(("cookieAuth" = [])),
    params(IdParam),
    request_body = AuthorCURequest,
    responses((status = 200, body = AuthorResponse)),
    tag = "Author"
)]
pub async fn update_author(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(id): ValidatedPath<i64>,
    ValidatedJson(body): ValidatedJson<AuthorCURequest>,
) -> Result<BaseResponse<AuthorResponse>, AppError> {

    let author = service::update(
        &state.db,
        id,
        body.name,
        body.avatar_path,
        lang
    ).await?;

    Ok(
        BaseResponse::success(
            mapper::to_response(author)
        )
    )
}


#[utoipa::path(
    get,
    path = "/v1/admin/author",
    security(("cookieAuth" = [])),
    params(PaginationParams),
    responses((status = 200, body = PagingResponse<AuthorResponse>)),
    tag = "Author"
)]
pub async fn page_author(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<PaginationParams>,
) -> Result<BaseResponse<PagingResponse<AuthorResponse>>, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    
    let authors = service::page(
        &state.db,
        page,
        limit,
        params.search
    ).await?;
    
    Ok(
        BaseResponse::success(
            PagingResponse::new(
                authors.0.into_iter().map(mapper::to_response).collect(),
                authors.1,
                page,
                limit
            )
        )
    )
}

#[utoipa::path(
    get,
    path = "/v1/admin/author/{id}",
    security(("cookieAuth" = [])),
    params(IdParam),
    responses((status = 200, body = AuthorResponse)),
    tag = "Author"
)]
pub async fn get_author(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(id): ValidatedPath<i64>
) -> Result<BaseResponse<AuthorResponse>, AppError> {
    let author = service::get(
        &state.db,
        id,
        lang
    ).await?;

    Ok(
        BaseResponse::success(mapper::to_response(author))
    )
}

#[utoipa::path(
    delete,
    path = "/v1/admin/author/{id}",
    security(("cookieAuth" = [])),
    params(IdParam),
    tag = "Author"
)]
pub async fn delete_author(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(id): ValidatedPath<i64>
) -> Result<BaseResponse<()>, AppError> {
    service::delete(
        &state.db,
        id,
        lang
    ).await?;
    Ok(BaseResponse::success(()))
}