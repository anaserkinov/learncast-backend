use axum::http::{StatusCode};
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};


#[derive(Serialize)]
pub struct BaseResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
    #[serde(with = "time::serde::rfc3339")]
    time: OffsetDateTime
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FileResponse {
    pub path: String,
    pub size: i64,
    pub duration: i64
}

#[derive(Deserialize, IntoParams)]
pub struct IdParam {
    pub id: i64
}

#[derive(Deserialize, IntoParams)]
pub struct ClientSnipIdParam {
    pub client_snip_id: String
}

#[derive(Deserialize, IntoParams)]
pub struct TopicIdParam {
    pub topic_id: i64
}

#[derive(Deserialize, IntoParams)]
pub struct LessonIdParam {
    pub lesson_id: i64
}

#[derive(Deserialize, IntoParams)]
pub struct UploadUrlParam {
    pub file_name: String,
    pub file_length: i64,
    pub mime_type: String
}

#[derive(Serialize, ToSchema)]
pub struct UploadUrlResponse {
    pub upload_url: String,
    pub file_key: String
}

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeletedParams {
    #[serde(with = "time::serde::rfc3339")]
    pub since: OffsetDateTime
}

#[derive(Serialize, ToSchema)]
pub struct DeletedResponse {
    pub id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub deleted_at: OffsetDateTime
}

impl<T> IntoResponse for BaseResponse<T>
where T: Serialize + Send,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl<T> BaseResponse<T> {
    pub fn empty() -> Self {
        Self {
            code: 0,
            message: "success".into(),
            data: None,
            time: OffsetDateTime::now_utc()
        }
    }
    
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".into(),
            data: Some(data),
            time: OffsetDateTime::now_utc(),
        }
    }

    pub fn error(
        code: i32,
        message: &str,
        data: Option<T>,
    ) -> Self {
        Self {
            code,
            message: message.to_string(),
            data,
            time: OffsetDateTime::now_utc()
        }
    }
}