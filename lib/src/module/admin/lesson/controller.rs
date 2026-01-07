use crate::extractor::accept_language::AcceptLanguage;
use crate::module::admin::lesson::dto::{LessonCURequest, LessonResponse};
use crate::module::admin::lesson::mapper;
use crate::module::common::base::{BaseResponse, IdParam, TopicIdParam};
use crate::module::common::lesson::service;
use crate::state::AppState;
use crate::utils::extractors::{ValidatedJson, ValidatedPath, ValidatedQuery};
use axum::extract::State;
use crate::error::AppError;
use crate::module::admin::lesson::dto::LessonPaginationParams;
use crate::module::common::paging::PagingResponse;

#[utoipa::path(
    post,
    path = "/v1/admin/lesson",
    security(("bearerAuth" = [])),
    params(TopicIdParam),
    request_body = LessonCURequest,
    responses((status = 200, body = LessonResponse)),
    tag = "Lesson"
)]
pub async fn create_lesson(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedJson(body): ValidatedJson<LessonCURequest>,
) -> Result<BaseResponse<LessonResponse>, AppError> {
    
    let topic = service::create(
        &state.db,
        &state.s3_client,
        body.author_id,
        body.topic_id,
        body.title,
        body.description,
        body.cover_image_path,
        body.audio_path,
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
    path = "/v1/admin/lesson/{id}",
    security(("bearerAuth" = [])),
    params(IdParam),
    request_body = LessonCURequest,
    responses((status = 200, body = LessonResponse)),
    tag = "Lesson"
)]
pub async fn update_lesson(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedPath(id): ValidatedPath<i64>,
    ValidatedJson(body): ValidatedJson<LessonCURequest>,
) -> Result<BaseResponse<LessonResponse>, AppError> {

    let topic = service::update(
        &state.db,
        &state.s3_client,
        id,
        body.title,
        body.description,
        body.cover_image_path,
        body.audio_path,
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
    path = "/v1/admin/lesson",
    security(("bearerAuth" = [])),
    params(LessonPaginationParams),
    responses((status = 200, body = PagingResponse<LessonResponse>)),
    tag = "Lesson"
)]
pub async fn page_lesson(
    State(state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<LessonPaginationParams>,
) -> Result<BaseResponse<PagingResponse<LessonResponse>>, AppError> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);

    let topics = service::page(
        &state.db,
        page,
        limit,
        params.author_id, 
        params.topic_id,
        params.search,
        None,
        None,
        None,
        None,
        None
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
    path = "/v1/admin/lesson/{id}",
    security(("bearerAuth" = [])),
    params(IdParam),
    responses((status = 200, body = LessonResponse)),
    tag = "Lesson"
)]
pub async fn get_lesson(
    State(state): State<AppState>,
    ValidatedPath(id): ValidatedPath<i64>,
    AcceptLanguage(lang): AcceptLanguage
) -> Result<BaseResponse<LessonResponse>, AppError> {
    let lesson = service::get(
        &state.db,
        id,
        lang
    ).await?;

    Ok(
        BaseResponse::success(mapper::to_response(lesson))
    )
}

#[utoipa::path(
    delete,
    path = "/v1/admin/lesson/{id}",
    security(("bearerAuth" = [])),
    params(IdParam),
    tag = "Lesson"
)]
pub async fn delete_lesson(
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