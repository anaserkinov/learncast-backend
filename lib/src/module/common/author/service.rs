use crate::db::author::entity::{AuthorEntity, AuthorInput};
use crate::error::author::AuthorError;
use crate::error::AppError;
use crate::{db, utils};
use fluent_templates::LanguageIdentifier;
use sqlx::PgPool;
use time::OffsetDateTime;
use crate::module::user::author::dto::AuthorCursor;

pub async fn create(
    db: &PgPool,
    name: String,
    avatar_path: Option<String>,
) -> Result<AuthorEntity, AppError> {
    let author = AuthorInput {
        name,
        avatar_path
    };
    let entity = db::author::repo::insert(
        db,
        author
    ).await?;

    Ok(entity)
}

pub async fn update(
    db: &PgPool,
    id: i64,
    name: String,
    avatar_path: Option<String>,
    lang: LanguageIdentifier
) -> Result<AuthorEntity, AppError> {
    let author = AuthorInput {
        name,
        avatar_path
    };
    let entity = db::author::repo::update(
        db,
        id,
        author
    ).await?.ok_or(AppError::NotFound(lang))?;

    Ok(entity)
}


pub async fn page(
    db: &PgPool,
    page: u32,
    limit: u32,
    search: Option<String>
) -> Result<(Vec<AuthorEntity>, u64), AppError>{
    let offset = (page - 1) * limit;

    let items = db::author::repo::page(
        db,
        limit,
        offset,
        &search
    ).await?;

    let total = db::author::repo::count(
        db,
        &search
    ).await?;
    
    Ok(
        (items, total as u64)
    )
}

pub async fn page_cursor(
    db: &PgPool,
    limit: u32,
    cursor: Option<String>,
    search: Option<String>
) -> Result<(Vec<AuthorEntity>, Option<String>), AppError>{
    let mut items = db::author::repo::page_cursor(
        db,
        limit + 1,
        utils::cursor::decode(cursor),
        &search
    ).await?;

    let next_cursor = if items.len() == (limit + 1) as usize {
        items.remove(limit as usize);
        let last = items.last().unwrap();
        utils::cursor::encode(
            AuthorCursor {
                id: last.id,
                lesson_count: last.lesson_count,
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
) -> Result<AuthorEntity, AppError>{
    let author = db::author::repo::get_by_id(
        db, id
    ).await?.ok_or(AppError::NotFound(lang))?;
    Ok(author)
}

pub async fn delete(
    db: &PgPool,
    id: i64,
    lang: LanguageIdentifier
) -> Result<(), AppError>{
    let mut tx = db.begin().await?;
    let author = db::author::repo::delete(
        &mut tx, id
    ).await?;

    if let Some(author) = author && author.lesson_count != 0{
        return Err(AuthorError::AuthorHasLesson(lang).into())
    }
    tx.commit().await?;
    Ok(())
}

pub async fn deleted(
    db: &PgPool,
    since: OffsetDateTime
) -> Result<Vec<(i64, OffsetDateTime)>, AppError>{
    let deleted_since = db::author::repo::deleted(db, since).await?;
    Ok(deleted_since)
}
