use crate::{db, utils};
use crate::db::snip::entity::{SnipEntity, SnipEntityWithLesson, SnipInput};
use crate::module::user::snip::dto::{QuerySort, SnipCursor};
use crate::module::common::paging::QueryOrder;
use anyhow::Result;
use fluent_templates::LanguageIdentifier;
use sqlx::PgPool;
use time::OffsetDateTime;
use crate::error::AppError;
use crate::error::snip::SnipError;

pub async fn create(
    db: &PgPool,
    client_snip_id: String,
    lesson_id: i64,
    user_id: i64,
    start_ms: i64,
    end_ms: i64,
    note_text: Option<String>,
    created_at: OffsetDateTime,
    lang: LanguageIdentifier
) -> Result<SnipEntityWithLesson> {
    let lesson = db::lesson::repo::get_with_author_topic_by_id(db, lesson_id)
        .await?
        .ok_or(AppError::NotFound(lang.clone()))?.lesson;
    
    let snip = SnipInput {
        client_snip_id,
        author_id: lesson.author_id,
        topic_id: lesson.topic_id,
        lesson_id,
        user_id,
        start_ms,
        end_ms,
        note_text,
        created_at
    };
    let mut tx = db.begin().await?;
    let insert_result = db::snip::repo::insert(
        &mut tx,
        snip
    ).await?;
    if insert_result.1 {
        db::lesson::repo::increase_snip_count(
            &mut tx,
            lesson_id
        ).await?;
        if let Some(topic_id) = lesson.topic_id {
            db::topic::repo::increase_snip_count(
                &mut tx,
                topic_id
            ).await?;
        }
    }
    tx.commit().await?;
    
    let entity = db::snip::repo::get_by_id(
        db,
        insert_result.0
    ).await?
        .ok_or(AppError::NotFound(lang))?;
    
    Ok(entity)
}

pub async fn update(
    db: &PgPool,
    client_snip_id: String,
    user_id: i64,
    start_ms: i64,
    end_ms: i64,
    note_text: Option<String>,
    lang: LanguageIdentifier
) -> Result<SnipEntityWithLesson> {
    let snip = SnipInput {
        client_snip_id: client_snip_id.clone(),
        author_id: 1,
        topic_id: None,
        lesson_id: 0,
        user_id: 0,
        start_ms,
        end_ms,
        note_text,
        created_at: OffsetDateTime::now_utc(),
    };

    let mut tx = db.begin().await?;
    let updated = db::snip::repo::update(
        &mut tx,
        client_snip_id,
        snip
    ).await?;
    if updated.user_id != user_id { 
        return Err(SnipError::SnipNotOwnedUpdate(lang).into());
    }
    tx.commit().await?;
    
    let entity = db::snip::repo::get_by_id(
        db,
        updated.id
    ).await?.ok_or(AppError::NotFound(lang))?;

    Ok(entity)
}

pub async fn delete(
    db: &PgPool,
    client_snip_id: String,
    user_id: i64,
    lang: LanguageIdentifier
) -> Result<Option<SnipEntity>> {
    let mut tx = db.begin().await?;
    let entity = db::snip::repo::delete(&mut tx, client_snip_id)
        .await?;
    if let Some(entity) = &entity && entity.user_id != user_id {
        return Err(SnipError::SnipNotOwnedDelete(lang).into());
    }
    tx.commit().await?;
    Ok(entity)
}

pub async fn deleted(
    db: &PgPool,
    since: OffsetDateTime
) -> std::result::Result<Vec<(i64, OffsetDateTime)>, AppError> {
    let deleted_since = db::snip::repo::deleted(db, since).await?;
    Ok(deleted_since)
}

pub async fn page(
    db: &PgPool,
    limit: u32,
    cursor: Option<String>,
    user_id: i64,
    lesson_id: Option<i64>,
    search: Option<String>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>
) -> Result<(Vec<SnipEntityWithLesson>, Option<String>)> {
    let mut items = db::snip::repo::page(
        db,
        limit + 1,
        utils::cursor::decode(cursor),
        user_id,
        lesson_id,
        &search,
        sort,
        order
    ).await?;

    let next_cursor = if items.len() == (limit + 1) as usize {
        items.remove(limit as usize);
        let last = items.last().unwrap();
        utils::cursor::encode(
            SnipCursor {
                id: last.snip.id,
                created_at: last.snip.created_at,
            }
        )
    } else { None };

    Ok(
        (items, next_cursor)
    )
}

pub async fn count(
    db: &PgPool,
    user_id: i64,
    lesson_id: i64
) -> Result<i64> {
    let count = db::snip::repo::count(db, user_id, lesson_id).await?;

    Ok(count)
}

