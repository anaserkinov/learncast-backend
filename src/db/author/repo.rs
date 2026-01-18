use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use time::OffsetDateTime;
use crate::db::author::entity::{AuthorEntity, AuthorInput};
use crate::module::user::author::dto::AuthorCursor;

pub async fn insert(db: &PgPool, topic: AuthorInput) -> Result<AuthorEntity, sqlx::Error> {
    sqlx::query_as::<_, AuthorEntity>(
        r#"
            INSERT INTO author (name, avatar_path)
            VALUES ($1, $2)
            RETURNING *
            "#
    )
        .bind(topic.name)
        .bind(topic.avatar_path)
        .fetch_one(db)
        .await
}

pub async fn update(db: &PgPool, id: i64, topic: AuthorInput) -> Result<Option<AuthorEntity>, sqlx::Error> {
    sqlx::query_as::<_, AuthorEntity>(
        r#"
        UPDATE author
        SET name = $1,
            avatar_path = $2
        WHERE id = $3
        RETURNING *
        "#
    )
        .bind(&topic.name)
        .bind(&topic.avatar_path)
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn delete(
    connection: &mut PgConnection,
    id: i64
) -> Result<Option<AuthorEntity>, sqlx::Error> {
    let result = sqlx::query_as::<_, AuthorEntity>(
        r#"
               UPDATE author SET deleted_at = NOW() WHERE id = $1
               RETURNING *
            "#
    )
        .bind(id)
        .fetch_optional(connection)
        .await?;
    Ok(result)
}

pub async fn deleted(
    db: &PgPool,
    since: OffsetDateTime
) -> Result<Vec<(i64, OffsetDateTime)>, sqlx::Error> {
    let result = sqlx::query_as::<_, (i64, OffsetDateTime)>(
        r#"
               SELECT id, deleted_at FROM author WHERE deleted_at IS NOT NULL AND deleted_at >= $1
            "#
    )
        .bind(since)
        .fetch_all(db)
        .await?;
    Ok(result)
}

fn build_query(
    query: &mut QueryBuilder<Postgres>,
    search: &Option<String>
) -> bool{
    let mut has_where = false;

    query.push(" WHERE author.deleted_at IS NULL");
    has_where = true;
    
    if let Some(search) = search && !search.is_empty() {
        query.push(if has_where { " AND " } else { " WHERE " })
            .push("name ILIKE ").push_bind(format!("{}%", search));
        has_where = true;
    }
    
    has_where
}

pub async fn page(
    db: &PgPool,
    limit: u32,
    offset: u32,
    search: &Option<String>
) -> Result<Vec<AuthorEntity>, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT author.*");
    query.push(" FROM author");

    build_query(&mut query, search);

    query.push(" ORDER BY lesson_count DESC");

    query
        .push(" LIMIT ").push_bind(limit as i32)
        .push(" OFFSET ").push_bind(offset as i32)
        .build_query_as::<AuthorEntity>()
        .fetch_all(db).await
}

pub async fn page_cursor(
    db: &PgPool,
    limit: u32,
    cursor: Option<AuthorCursor>,
    search: &Option<String>
) -> Result<Vec<AuthorEntity>, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT author.*");
    query.push(" FROM author");

    let has_where = build_query(&mut query, search);

    if let Some(cursor) = cursor {
        query.push(if has_where { " AND " } else { " WHERE " })
            .push("(lesson_count, id) > (")
            .push_bind(cursor.lesson_count).push(",").push_bind(cursor.id).push(")");
    }
    
    query.push(" ORDER BY lesson_count DESC, id DESC");

    query
        .push(" LIMIT ").push_bind(limit as i32)
        .build_query_as::<AuthorEntity>()
        .fetch_all(db).await
}

pub async fn count(
    db: &PgPool,
    search: &Option<String>,
) -> Result<i64, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(id) FROM author");
    build_query(
        &mut query,
        search
    );
    Ok(
        query
            .build_query_as::<(i64,)>()
            .fetch_one(db).await?.0
    )
}

pub async fn get_by_id(db: &PgPool, id: i64) -> Result<Option<AuthorEntity>, sqlx::Error> {
    Ok(sqlx::query_as::<_, AuthorEntity>(
        r#"
             SELECT * FROM author
             WHERE id = $1
             "#,
    )
        .bind(id)
        .fetch_optional(db)
        .await?)
}

pub async fn get_by_lesson_id(db: &PgPool, id: i64) -> Result<Option<AuthorEntity>, sqlx::Error> {
    Ok(sqlx::query_as::<_, AuthorEntity>(
        r#"
             SELECT author.* FROM author
             JOIN lesson ON lesson.author_id = author.id
             WHERE lesson.id = $1
            "#,
    )
        .bind(id)
        .fetch_optional(db)
        .await?)
}

pub async fn update_stats(
    connection: &mut PgConnection,
    author_id: i64,
    lesson_delta: i64
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE author
        SET 
            lesson_count = lesson_count + $1
        WHERE id = $2
        "#
    )
        .bind(lesson_delta)
        .bind(author_id)
        .execute(connection)
        .await?;

    Ok(())
}