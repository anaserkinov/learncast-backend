use crate::db::snip::entity::{SnipEntity, SnipEntityWithLesson, SnipInput};
use crate::module::common::paging::QueryOrder;
use crate::module::user::snip::dto::{QuerySort, SnipCursor};
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use time::OffsetDateTime;

pub async fn insert(
    connection: &mut PgConnection,
    snip: SnipInput
) -> Result<(i64, bool), sqlx::Error> {
    Ok(
        sqlx::query_as::<_, (i64,bool)>(
            r#"
            INSERT INTO snip (client_snip_id, author_id, topic_id, lesson_id, user_id, start_ms, end_ms, note_text, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (client_snip_id) DO UPDATE
                SET client_snip_id = EXCLUDED.client_snip_id
            RETURNING id, (xmax = 0) AS inserted;
            "#,
        )
            .bind(snip.client_snip_id)
            .bind(snip.author_id)
            .bind(snip.topic_id)
            .bind(snip.lesson_id)
            .bind(snip.user_id)
            .bind(snip.start_ms)
            .bind(snip.end_ms)
            .bind(snip.note_text)
            .bind(snip.created_at)
            .fetch_one(connection)
            .await?
    )
}

pub async fn update(
    connection: &mut PgConnection,
    client_snip_id: String,
    snip: SnipInput,
) -> Result<SnipEntity, sqlx::Error> {
    Ok(
        sqlx::query_as::<_, SnipEntity>(
            r#"
        UPDATE snip
        SET start_ms = $1,
            end_ms = $2,
            note_text = $3
        WHERE client_snip_id = $4
        RETURNING *
        "#,
        ).bind(snip.start_ms)
            .bind(snip.end_ms)
            .bind(snip.note_text)
            .bind(client_snip_id)
            .fetch_one(connection)
            .await?
    )
}

pub async fn delete(
    connection: &mut PgConnection,
    client_snip_id: String
) -> Result<Option<SnipEntity>, sqlx::Error> {
    let result = sqlx::query_as::<_, SnipEntity>(
        r#"
               UPDATE snip SET deleted_at = NOW() WHERE client_snip_id = $1
               RETURNING *
            "#
    )
        .bind(client_snip_id)
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
               SELECT id, deleted_at FROM snip WHERE deleted_at IS NOT NULL AND deleted_at >= $1
            "#
    )
        .bind(since)
        .fetch_all(db)
        .await?;
    Ok(result)
}

fn build_query(
    query: &mut QueryBuilder<Postgres>,
    user_id: i64,
    lesson_id: Option<i64>,
    search: &Option<String>
){
    query.push(" JOIN lesson ON lesson.id = snip.lesson_id AND lesson.deleted_at IS NULL")
        .push(" JOIN author ON author.id = snip.author_id")
        .push(" LEFT JOIN topic ON topic.id = snip.topic_id")
        .push(" WHERE snip.deleted_at IS NULL AND user_id = ").push_bind(user_id);

    if let Some(lesson_id) = lesson_id {
        query.push(" AND snip.lesson_id = ").push_bind(lesson_id);
    }

    if let Some(search) = search {
        query.push(" AND (snip.note_text ILIKE ").push_bind(format!("{}%", search))
            .push(" OR lesson.title ILIKE ").push_bind(format!("{}%", search));
    }
}


pub async fn page(
    db: &PgPool,
    limit: u32,
    cursor: Option<SnipCursor>,
    user_id: i64,
    lesson_id: Option<i64>,
    search: &Option<String>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>
) -> Result<Vec<SnipEntityWithLesson>, sqlx::Error> {
    let (order, order_sign) = if order == Some(QueryOrder::Asc) { ("ASC", ">") }
    else { ("DESC", "<") };

    let mut query = QueryBuilder::<Postgres>::new("SELECT");

    query.push(r#"
        snip.*,

        lesson.title AS lesson_title,
        lesson.description AS lesson_description,
        lesson.cover_image_path AS lesson_cover_image_path,
        lesson.audio_path AS lesson_audio_path,
        lesson.duration AS lesson_duration,
        lesson.file_size AS lesson_file_size,
        lesson.listen_count AS lesson_listen_count,
        lesson.snip_count AS lesson_snip_count,
        lesson.created_at AS lesson_created_at,

        author.name AS author_name,
        author.avatar_path AS author_avatar_path,
        author.created_at AS author_created_at,
        author.lesson_count AS author_lesson_count,

        topic.title AS topic_title,
        topic.description AS topic_description,
        topic.cover_image_path AS topic_cover_image_path,
        topic.created_at AS topic_created_at,
        topic.lesson_count AS topic_lesson_count,
        topic.total_duration AS topic_total_duration,
        topic.snip_count AS topic_snip_count
    "#);

    query.push(" FROM snip");

    build_query(&mut query, user_id, lesson_id, search);

    if let Some(cursor) = cursor {
        query.push(" AND ");
        match sort {
            Some(QuerySort::CreatedAt) => {
                query.push("(snip.created_at,snip.id)").push(order_sign)
                    .push("(").push_bind(cursor.created_at).push(",").push_bind(cursor.id).push(")");
            }
            _ => {
                query.push("snip.id").push(order_sign)
                    .push_bind(cursor.id);
            }
        };
    }

    query.push(" ORDER BY ")
        .push(
            match sort {
                Some(QuerySort::CreatedAt) => { format!("snip.created_at {order}, snip.id {order}") }
                None => { format!("snip.id {order}") }
            }
        );

    query .push(" LIMIT ").push_bind(limit as i64)
        .build_query_as::<SnipEntityWithLesson>()
        .fetch_all(db).await
}


pub async fn count(
    db: &PgPool,
    user_id: i64,
    lesson_id: i64
) -> Result<i64, sqlx::Error> {
    Ok(
        sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM snip WHERE user_id = $1 AND lesson_id = $2
            "#,
        )
            .bind(user_id)
            .bind(lesson_id)
            .fetch_one(db).await?.0
    )
}


pub async fn get_by_id(
    db: &PgPool,
    id: i64
) -> Result<Option<SnipEntityWithLesson>, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT");

    query.push(r#"
        snip.*,

        lesson.title AS lesson_title,
        lesson.description AS lesson_description,
        lesson.cover_image_path AS lesson_cover_image_path,
        lesson.audio_path AS lesson_audio_path,
        lesson.duration AS lesson_duration,
        lesson.file_size AS lesson_file_size,
        lesson.listen_count AS lesson_listen_count,
        lesson.snip_count AS lesson_snip_count,
        lesson.created_at AS lesson_created_at,

        author.name AS author_name,
        author.avatar_path AS author_avatar_path,
        author.created_at AS author_created_at,
        author.lesson_count AS author_lesson_count,

        topic.title AS topic_title,
        topic.description AS topic_description,
        topic.cover_image_path AS topic_cover_image_path,
        topic.created_at AS topic_created_at,
        topic.lesson_count AS topic_lesson_count,
        topic.total_duration AS topic_total_duration,
        topic.snip_count AS topic_snip_count
    "#).push(" FROM snip")
        .push(" JOIN lesson ON lesson.id = snip.lesson_id AND lesson.deleted_at IS NULL")
        .push(" JOIN author ON author.id = snip.author_id")
        .push(" LEFT JOIN topic ON topic.id = snip.topic_id")
        .push(" WHERE snip.id = ").push_bind(id);

    query
        .build_query_as::<SnipEntityWithLesson>()
        .fetch_optional(db).await
}