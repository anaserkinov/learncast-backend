use crate::error::AppError;
use crate::extractor::accept_language::AcceptLanguage;
use crate::module::common::base::{BaseResponse, UploadUrlParam, UploadUrlResponse};
use crate::state::AppState;
use crate::utils::extractors::{ValidatedPath, ValidatedQuery};
use crate::utils::CONFIG;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::extract::{Multipart, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::Duration;
use axum::http::{header, HeaderValue};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::db;

/// Just a schema for axum native multipart
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct UploadForm {
    name: String,
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

fn detect_file_kind(bytes: &[u8]) -> Option<(&str, &str)> {
    let file = infer::get(bytes)?;

    let category = match file.mime_type() {
        m if m.starts_with("image/") => "image",
        m if m.starts_with("audio/") => "audio",
        _ => return None,
    };

    Some((
        category,
        file.extension()
    ))
}

#[utoipa::path(
    post,
    path = "/v1/file",
    security(("bearerAuth" = [])),
    request_body(content = UploadForm, content_type = "multipart/form-data"),
    responses((status = 200, body = String)),
    tag = "File"
)]
pub async fn upload(
    AcceptLanguage(lang): AcceptLanguage,
    mut multipart: Multipart,
) -> Result<BaseResponse<String>, AppError> {
    let mut audio_path = None;

    if let Ok(field) = multipart.next_field().await {
        if let Some(field) = field && let Some(name) = field.name() && name == "file" {
            let data = field.bytes().await.unwrap();
            let file_kind = detect_file_kind(&data).ok_or(
                AppError::UnsupportedFileType(lang.clone())
            )?;

            if file_kind.0 == "audio" {
                return Err(AppError::UnsupportedFileType(lang))
            }

            let mut hasher = Sha256::new();
            hasher.update(&data);

            let hash = format!("{:x}", hasher.finalize());
            let filename = format!("{}.{}", hash, file_kind.1);
            audio_path = Some(format!("{}/{}", file_kind.0, filename));

            let path_str = format!("uploads/{}", audio_path.as_ref().unwrap());
            let path = Path::new(&path_str);

            if let Some(parent) = path.parent() && let Err(_) = fs::create_dir_all(parent) {
                return Err(AppError::Internal(lang))
            }

            if let Err(_) = fs::write(&path, &data) {
                return Err(AppError::Internal(lang))
            }
        }
    }
    
    if let Some(path) = audio_path {
        Ok(BaseResponse::success(path))
    } else {
        Err(AppError::NotFound(lang))
    }
}

#[utoipa::path(
    get,
    path = "/v1/file/upload-url",
    security(("bearerAuth" = [])),
    params(UploadUrlParam),
    responses((status = 200, body = UploadUrlResponse)),
    tag = "File"
)]
pub async fn upload_url(
    State(state): State<AppState>,
    AcceptLanguage(lang): AcceptLanguage,
    ValidatedQuery(params): ValidatedQuery<UploadUrlParam>
) -> Result<BaseResponse<UploadUrlResponse>, AppError> {

    if params.file_length > 100 * 1024 * 1024 {
        return Err(AppError::FileTooLarge(lang))
    }

    let is_audio = match params.mime_type.split_once('/') {
        Some((type_part, _subtype)) => type_part == "audio",
        None => false,
    };

    if !is_audio {
        return Err(AppError::UnsupportedFileType(lang))
    }

    let ext = params.file_name.rsplit('.').next().and_then(|ext| {
        if ext == params.file_name || ext.to_lowercase() != "mp3" {
            None
        } else {
            Some(ext)
        }
    }).ok_or(AppError::UnsupportedFileType(lang.clone()))?;


    let filename = format!("{}.{}", Uuid::new_v4().to_string(), ext);
    let audio_path = format!("{}/{}", "audio", filename);

    let expires_in = PresigningConfig::expires_in(Duration::from_mins(1))
        .map_err(|_| AppError::Internal(lang.clone()))?;

    let presigned_request = state.s3_client
        .put_object()
        .bucket(CONFIG.r2_bucket_name.clone())
        .key(audio_path.clone())
        .content_length(params.file_length)
        .content_type(params.mime_type)
        .presigned(expires_in)
        .await
        .map_err(|_| { AppError::Internal(lang.clone()) })?;

    Ok(
        BaseResponse::success(
            UploadUrlResponse{
                upload_url: presigned_request.uri().into(),
                file_key: audio_path
            }
        )
    )
}

#[utoipa::path(
    get,
    path = "/v1/file/{file_path}",
    security(("bearerAuth" = [])),
    params(("file_path" = String, Path)),
    responses((status = 200, content_type = "application/octet-stream")),
    tag = "File"
)]
pub async fn download_file(
    State(state): State<AppState>,
    ValidatedPath(file_path): ValidatedPath<String>,
    AcceptLanguage(lang): AcceptLanguage
) -> Result<Response, AppError> {

    let ext = file_path.rsplit('.').next().and_then(|ext| {
        if ext == file_path {
            None
        } else {
            Some(ext)
        }
    }).ok_or(AppError::UnsupportedFileType(lang.clone()))?;

    if ext == "mp3" {

        // Default to 5 minutes if duration is unavailable.
        // This can happen while a lesson is being created and an admin requests a preview.
        let audio_duration_mins = db::lesson::repo::get_duration(
            &state.db,
            file_path.clone()
        ).await?.unwrap_or(5 * 60 * 1000)/(60 * 1000);

        let expires_in = PresigningConfig::expires_in(
            Duration::from_mins((((audio_duration_mins / 10) + 1) * 10) as u64)
        ).map_err(|_| AppError::Internal(lang.clone()))?;
        let presigned_request = state.s3_client
            .get_object()
            .bucket(CONFIG.r2_bucket_name.clone())
            .key(file_path)
            .presigned(expires_in)
            .await
            .map_err(|e| { eprintln!("{:?}", e);
                AppError::Internal(lang.clone()) })?;

        let mut response = Redirect::temporary(
            &presigned_request.uri().to_string()
        ).into_response();
        let headers = response.headers_mut();
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache, no-store, must-revalidate"));
        headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
        headers.insert(header::EXPIRES, HeaderValue::from_static("0"));

        return Ok(response);
    }

    let path = format!("uploads/{}", file_path);

    if !Path::new(&path).exists() {
        return Err(AppError::NotFound(lang));
    }

    match fs::read(&path) {
        Ok(bytes) => {
            let mime_type = infer::get(&bytes)
                .map(|t| t.mime_type())
                .unwrap_or("application/octet-stream");

            let mut response = Response::new(bytes.into());
            let headers = response.headers_mut();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(mime_type).unwrap(), );
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file_path)).unwrap(),
            );

            Ok(response)
        }
        Err(_) => Err(AppError::Internal(lang))
    }
}

pub async fn download_image(
    State(state): State<AppState>,
    ValidatedPath(file_path): ValidatedPath<String>,
    AcceptLanguage(lang): AcceptLanguage
) -> Result<Response, AppError> {
    download_file(
        State(state),
        ValidatedPath(format!("image/{}", file_path)),
        AcceptLanguage(lang)
    ).await
}