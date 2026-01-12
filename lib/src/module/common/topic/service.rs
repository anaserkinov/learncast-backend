use crate::db::topic::entity::{TopicEntity, TopicInput, TopicWithAuthor};
use crate::error::topic::TopicError;
use crate::error::AppError;
use crate::module::common::enums::UserProgressStatus;
use crate::module::common::paging::QueryOrder;
use crate::module::common::topic::dto::QuerySort;
use crate::module::user::topic::dto::TopicCursor;
use crate::{db, utils};
use anyhow::Result;
use fluent_templates::LanguageIdentifier;
use sqlx::PgPool;
use time::OffsetDateTime;

pub async fn create(
    db: &PgPool,
    title: String,
    description: Option<String>,
    cover_image_path: Option<String>
) -> Result<TopicEntity> {
    let topic = TopicInput {
        title,
        description,
        cover_image_path
    };
    let entity = db::topic::repo::insert(
        db,
        topic
    ).await?;

    Ok(entity)
}

pub async fn update(
    db: &PgPool,
    id: i64,
    title: String,
    description: Option<String>,
    cover_image_path: Option<String>,
    lang: LanguageIdentifier
) -> Result<TopicEntity> {
    let topic = TopicInput {
        title,
        description,
        cover_image_path
    };
    let entity = db::topic::repo::update(
        db,
        id,
        topic
    ).await?.ok_or(AppError::NotFound(lang))?;

    Ok(entity)
}


pub async fn page(
    db: &PgPool,
    page: u32,
    limit: u32,
    search: Option<String>,
    author_id: Option<i64>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>
) -> Result<(Vec<TopicEntity>, u64)>{
    let offset = (page - 1) * limit;

    let items = db::topic::repo::page(
        db,
        limit,
        offset,
        &search,
        author_id,
        sort,
        order
    ).await?;

    let total = db::topic::repo::count(
        db,
        &search,
        author_id
    ).await?;
    
    Ok(
        (items, total as u64)
    )
}

pub async fn page_with_author(
    db: &PgPool,
    limit: u32,
    cursor: Option<String>,
    search: Option<String>,
    user_id: Option<i64>,
    author_id: Option<i64>,
    status: Option<UserProgressStatus>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>
) -> Result<(Vec<TopicWithAuthor>, Option<String>)> {
    let mut items = db::topic::repo::page_with_author(
        db,
        limit + 1,
        utils::cursor::decode(cursor),
        &search,
        user_id,
        author_id,
        &status,
        sort,
        order
    ).await?;

    let next_cursor = if items.len() == (limit + 1) as usize {
        items.remove(limit as usize);
        let last = items.last().unwrap();
        utils::cursor::encode(
            TopicCursor {
                id: last.topic.id,
                snip_count: last.topic.snip_count,
                created_at: last.topic.created_at
            }
        )
    } else { None };

    Ok(
        (items, next_cursor)
    )
}

pub async fn get(
    db: &PgPool,
    id: i64,
    lang: LanguageIdentifier
) -> Result<TopicEntity>{
    let topic = db::topic::repo::get_by_id(
        db, id
    ).await?.ok_or(AppError::NotFound(lang))?;
    Ok(topic)
}

pub async fn delete(
    db: &PgPool,
    id: i64,
    lang: LanguageIdentifier
) -> Result<()>{
    let mut tx = db.begin().await?;
    let topic = db::topic::repo::delete(
        &mut tx, id
    ).await?;
    
    if let Some(topic) = topic && topic.lesson_count != 0{ 
        return Err(TopicError::TopicHasLesson(lang).into())
    }
    tx.commit().await?;
    Ok(())
}

pub async fn deleted(
    db: &PgPool,
    since: OffsetDateTime
) -> std::result::Result<Vec<(i64, OffsetDateTime)>, AppError> {
    let deleted_since = db::topic::repo::deleted(db, since).await?;
    Ok(deleted_since)
}

