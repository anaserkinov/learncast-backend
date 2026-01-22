use crate::db::lesson::entity::{LessonInput, LessonProgressEntity, LessonWithAuthorTopic};
use crate::error::lesson::LessonError;
use crate::error::AppError;
use crate::module::common::enums::UserProgressStatus;
use crate::module::common::lesson::dto::QuerySort;
use crate::module::common::paging::QueryOrder;
use crate::module::user::lesson::dto::LessonCursor;
use crate::utils::CONFIG;
use crate::{db, utils};
use anyhow::Result;
use aws_sdk_s3 as s3;
use aws_sdk_s3::presigning::PresigningConfig;
use ffmpeg_light::probe;
use fluent_templates::LanguageIdentifier;
use sqlx::PgPool;
use std::time::Duration;
use time::OffsetDateTime;

async fn get_info(s3_client: &s3::Client, path: &str) -> Result<(i64, i64)> {
    let head = s3_client
        .head_object()
        .bucket(CONFIG.r2_bucket_name.clone())
        .key(path)
        .send()
        .await?;

    let size_bytes = head.content_length().unwrap_or_default();

    let expires_in = PresigningConfig::expires_in(Duration::from_secs(15))?;
    let presigned_request = s3_client
        .get_object()
        .bucket(CONFIG.r2_bucket_name.clone())
        .key(path)
        .presigned(expires_in)
        .await?;

    let info = probe(
        presigned_request.uri().to_string()
    )?;
    Ok(
        (
            size_bytes,
            info.duration().unwrap_or(Duration::from_secs(0)).as_millis() as i64
        )
    )
}

pub async fn create(
    db: &PgPool,
    s3_client: &s3::Client,
    author_id: i64,
    topic_id: Option<i64>,
    title: String,
    description: Option<String>,
    cover_image_path: Option<String>,
    audio_path: String,
    lang: LanguageIdentifier,
) -> Result<LessonWithAuthorTopic, AppError> {
    let info = get_info(s3_client, audio_path.as_str()).await?;

    let lesson = LessonInput {
        author_id,
        topic_id,
        title,
        description,
        cover_image_path,
        audio_path,
        duration: info.1,
        file_size: info.0,
    };

    if let Some(topic_id) = topic_id {
        let topic = db::topic::repo::get_by_id(db, topic_id).await?;
        if topic.is_none() || topic.unwrap().topic.author_id != author_id{
            return Err(AppError::NotFound(lang))
        }
    }

    let mut tx = db.begin().await?;
    let lesson_id = db::lesson::repo::insert(&mut tx, lesson).await?;
    if let Some(topic_id) = topic_id {
        db::topic::repo::update_stats(&mut tx, topic_id, 1, info.1).await?;
    }
    db::author::repo::update_stats(&mut tx, author_id, 1).await?;
    tx.commit().await?;
    let lesson = db::lesson::repo::get_with_author_topic_by_id(db, lesson_id)
        .await?
        .ok_or(AppError::NotFound(lang))?;

    Ok(lesson)
}

pub async fn update(
    db: &PgPool,
    s3_client: &s3::Client,
    id: i64,
    title: String,
    description: Option<String>,
    cover_image_path: Option<String>,
    audio_path: String,
    lang: LanguageIdentifier,
) -> Result<LessonWithAuthorTopic> {
    let old_lesson = db::lesson::repo::get_by_id(db, id)
        .await?.ok_or(AppError::NotFound(lang.clone()))?;

    let audio_updated = old_lesson.audio_path != audio_path;

    let info = if audio_updated {
        get_info(s3_client, audio_path.as_str()).await?
    } else {
        (old_lesson.file_size, old_lesson.duration)
    };

    let lesson = LessonInput {
        author_id: 1,
        topic_id: None,
        title,
        description,
        cover_image_path,
        audio_path,
        duration: info.1,
        file_size: info.0,
    };

    let mut tx = db.begin().await?;
    let lesson_id = db::lesson::repo::update(&mut tx, id, lesson)
        .await?
        .ok_or(AppError::NotFound(lang.clone()))?;
    let entity = db::lesson::repo::get_with_author_topic_by_id(db, lesson_id)
        .await?
        .ok_or(AppError::NotFound(lang))?;
    if let Some(topic_id) = entity.lesson.topic_id && audio_updated{
        db::topic::repo::update_stats(
            &mut tx,
            topic_id,
            0,
            entity.lesson.duration - old_lesson.duration,
        )
        .await?
    }
    tx.commit().await?;

    Ok(entity)
}

pub async fn page(
    db: &PgPool,
    page: u32,
    limit: u32,
    author_id: Option<i64>,
    topic_id: Option<i64>,
    search: Option<String>,
    user_id: Option<i64>,
    status: Option<UserProgressStatus>,
    favourite: Option<bool>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>,
) -> Result<(Vec<LessonWithAuthorTopic>, u64)> {
    let offset = (page - 1) * limit;

    let items = db::lesson::repo::page(
        db, limit, offset, author_id, topic_id, &search, user_id, &status, favourite, sort, order,
    )
    .await?;

    let total = db::lesson::repo::count(
        db, author_id, topic_id, &search, user_id, &status, favourite,
    )
    .await?;

    Ok((items, total as u64))
}

pub async fn page_cursor(
    db: &PgPool,
    limit: u32,
    cursor: Option<String>,
    author_id: Option<i64>,
    topic_id: Option<i64>,
    search: Option<String>,
    user_id: Option<i64>,
    status: Option<UserProgressStatus>,
    favourite: Option<bool>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>,
) -> Result<(Vec<LessonWithAuthorTopic>, Option<String>)> {
    let mut items = db::lesson::repo::page_cursor(
        db,
        limit,
        utils::cursor::decode(cursor),
        author_id,
        topic_id,
        &search,
        user_id,
        &status,
        favourite,
        sort,
        order,
    )
    .await?;

    let next_cursor = if items.len() == (limit + 1) as usize {
        items.remove(limit as usize);
        let last = items.last().unwrap();
        utils::cursor::encode(LessonCursor {
            id: last.lesson.id,
            snip_count: last.lesson.snip_count,
            created_at: last.lesson.created_at,
        })
    } else {
        None
    };

    Ok((items, next_cursor))
}

pub async fn get(db: &PgPool, id: i64, lang: LanguageIdentifier) -> Result<LessonWithAuthorTopic> {
    let lesson = db::lesson::repo::get_with_author_topic_by_id(db, id)
        .await?
        .ok_or(AppError::NotFound(lang))?;
    Ok(lesson)
}

pub async fn increase_listen_count(
    db: &PgPool,
    session_id: String,
    user_id: i64,
    lesson_id: i64,
    created_at: OffsetDateTime,
) -> Result<i64> {
    let mut tx = db.begin().await?;
    let result = db::lesson::repo::create_listen_session(
        &mut tx,
        session_id,
        user_id,
        lesson_id,
        created_at
    ).await?;
    if result.1 {
        db::lesson::repo::increase_listen_count(
            &mut tx,
            lesson_id
        ).await?;
    }
    tx.commit().await?;
    Ok(
        db::lesson::repo::get_listen_count(db, lesson_id).await?
    )
}

pub async fn update_progress(
    db: &PgPool,
    user_id: i64,
    lesson_id: i64,
    started_at: OffsetDateTime,
    last_position_ms: i64,
    status: Option<UserProgressStatus>,
    completed_at: Option<OffsetDateTime>,
    lang: LanguageIdentifier,
) -> Result<LessonProgressEntity> {
    let author = db::author::repo::get_by_lesson_id(db, lesson_id)
        .await?
        .ok_or(AppError::NotFound(lang))?;
    let topic = db::topic::repo::get_by_lesson_id(db, lesson_id).await?;
    let mut tx = db.begin().await?;
    let entity = db::lesson::repo::update_progress(
        &mut tx,
        user_id,
        author.id,
        topic.as_ref().map(|t| t.id),
        lesson_id,
        started_at,
        last_position_ms,
        status,
        completed_at,
    )
    .await?;
    if let Some(topic) = topic {
        db::topic::repo::update_progress(&mut tx, user_id, author.id, topic.id).await?;
    }
    tx.commit().await?;

    Ok(entity)
}

pub async fn delete(db: &PgPool, id: i64, lang: LanguageIdentifier) -> Result<()> {
    let mut tx = db.begin().await?;
    let entity = db::lesson::repo::delete(&mut tx, id).await?;

    if let Some(entity) = &entity
        && entity.listen_count >= 50
    {
        return Err(LessonError::LessonDeleteTooManyListens(lang).into());
    }

    if let Some(lesson) = entity {
        if let Some(topic_id) = lesson.topic_id {
            db::topic::repo::update_stats(
                &mut tx,
                topic_id,
                -1,
                -lesson.duration,
            )
            .await?;
        }
        db::author::repo::update_stats(&mut tx, lesson.author_id, -1).await?;
    }
    tx.commit().await?;

    Ok(())
}

pub async fn deleted(
    db: &PgPool,
    since: OffsetDateTime
) -> std::result::Result<Vec<(i64, OffsetDateTime)>, AppError> {
    let deleted_since = db::lesson::repo::deleted(db, since).await?;
    Ok(deleted_since)
}

pub async fn set_favourite(db: &PgPool, user_id: i64, lesson_id: i64) -> Result<()> {
    db::lesson::repo::set_favourite(db, user_id, lesson_id).await?;
    Ok(())
}

pub async fn remove_favourite(db: &PgPool, user_id: i64, lesson_id: i64) -> Result<()> {
    db::lesson::repo::remove_favourite(db, user_id, lesson_id).await?;
    Ok(())
}
