use crate::state::AppState;
use axum::extract::{State};
use crate::error::AppError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::module::admin::topic::{mapper};
use crate::module::admin::topic::dto::{TopicCURequest, PaginationParams, TopicResponse};
use crate::module::common::base::{BaseResponse, IdParam};
use crate::module::common::paging::PagingResponse;
use crate::module::common::topic::service;
use crate::utils::extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};

#[utoipa::path(
    post,
    path = "/v1/admin/topic",
    security(("cookieAuth" = [])),
    request_body = TopicCURequest,
    responses((status = 200, body = TopicResponse)),
    tag = "Topic"
)]
pub async fn create_topic(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedJson(body): ValidatedJson<TopicCURequest>,
) -> Result<BaseResponse<TopicResponse>, AppError> {
    
    let topic = service::create(
        &state.db,
        body.author_id,
        body.title,
        body.description,
        body.cover_image_path,
        lang
    ).await?;

    Ok(
        BaseResponse::success(
            mapper::to_response(topic)
        )
    )
}

#[utoipa::path(
    put,
    path = "/v1/admin/topic/{id}",
    security(("cookieAuth" = [])),
    params(IdParam),
    request_body = TopicCURequest,
    responses((status = 200, body = TopicResponse)),
    tag = "Topic"
)]
pub async fn update_topic(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(id): ValidatedPath<i64>,
    ValidatedJson(body): ValidatedJson<TopicCURequest>,
) -> Result<BaseResponse<TopicResponse>, AppError> {

    let topic = service::update(
        &state.db,
        id,
        body.title,
        body.description,
        body.cover_image_path,
        lang
    ).await?;
    
    Ok(
        BaseResponse::success(
            mapper::to_response(topic)
        )
    )
}


#[utoipa::path(
    get,
    path = "/v1/admin/topic",
    security(("cookieAuth" = [])),
    params(PaginationParams),
    responses((status = 200, body = PagingResponse<TopicResponse>)),
    tag = "Topic"
)]
pub async fn page_topic(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<PaginationParams>,
) -> Result<BaseResponse<PagingResponse<TopicResponse>>, AppError> {
    let page = params.page;
    let limit = params.limit;
    
    let topics = service::page(
        &state.db,
        page,
        limit,
        params.search,
        params.author_id,
        params.sort,
        params.order
    ).await?;
    
    Ok(
        BaseResponse::success(
            PagingResponse::new(
                topics.0.into_iter().map(mapper::to_response).collect(),
                topics.1,
                page,
                limit
            )
        )
    )
}

#[utoipa::path(
    get,
    path = "/v1/admin/topic/{id}",
    security(("cookieAuth" = [])),
    params(IdParam),
    responses((status = 200, body = TopicResponse)),
    tag = "Topic"
)]
pub async fn get_topic(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(id): ValidatedPath<i64>
) -> Result<BaseResponse<TopicResponse>, AppError> {
    let topic = service::get(
        &state.db,
        id,
        lang
    ).await?;

    Ok(
        BaseResponse::success(mapper::to_response(topic))
    )
}

#[utoipa::path(
    delete,
    path = "/v1/admin/topic/{id}",
    security(("cookieAuth" = [])),
    params(IdParam),
    tag = "Topic"
)]
pub async fn delete_topic(
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