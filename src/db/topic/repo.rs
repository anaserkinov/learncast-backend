use crate::db::topic::entity::{TopicEntity, TopicInput, TopicWithAuthor};
use crate::module::common::enums::UserProgressStatus;
use crate::module::common::paging::QueryOrder;
use crate::module::common::topic::dto::QuerySort;
use crate::module::user::topic::dto::TopicCursor;
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use time::OffsetDateTime;

pub async fn insert(db: &PgPool, topic: TopicInput) -> Result<TopicEntity, sqlx::Error> {
    sqlx::query_as::<_, TopicEntity>(
        r#"
            INSERT INTO topic (author_id, title, description, cover_image_path)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
    )
    .bind(topic.author_id)
    .bind(topic.title)
    .bind(topic.description)
    .bind(topic.cover_image_path)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: i64,
    topic: TopicInput,
) -> Result<Option<TopicEntity>, sqlx::Error> {
    sqlx::query_as::<_, TopicEntity>(
        r#"
        UPDATE topic
        SET title = $1,
            description = $2,
            cover_image_path = $3
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(topic.title)
    .bind(topic.description)
    .bind(topic.cover_image_path)
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn delete(
    connection: &mut PgConnection,
    id: i64,
) -> Result<Option<TopicEntity>, sqlx::Error> {
    let result = sqlx::query_as::<_, TopicEntity>(
        r#"
               UPDATE topic SET deleted_at = NOW() WHERE id = $1
               RETURNING *
            "#,
    )
    .bind(id)
    .fetch_optional(connection)
    .await?;
    Ok(result)
}

pub async fn deleted(
    db: &PgPool,
    since: OffsetDateTime,
) -> Result<Vec<(i64, OffsetDateTime)>, sqlx::Error> {
    let result = sqlx::query_as::<_, (i64, OffsetDateTime)>(
        r#"
               SELECT id, deleted_at FROM topic WHERE deleted_at IS NOT NULL AND deleted_at >= $1
            "#,
    )
    .bind(since)
    .fetch_all(db)
    .await?;
    Ok(result)
}

fn build_query(
    query: &mut QueryBuilder<Postgres>,
    search: &Option<String>,
    author_id: Option<i64>,
) -> bool {
    let mut has_where = false;

    query.push(" WHERE topic.deleted_at IS NULL");
    has_where = true;

    if let Some(author_id) = author_id {
        query
            .push(if has_where { " AND " } else { " WHERE " })
            .push("topic.author_id = ")
            .push_bind(author_id);
        has_where = true;
    }

    if let Some(search) = search {
        query
            .push(if has_where { " AND " } else { " WHERE " })
            .push("title ILIKE ")
            .push_bind(format!("{}%", search));
        has_where = true;
    }

    has_where
}

pub async fn page(
    db: &PgPool,
    limit: u32,
    offset: u32,
    search: &Option<String>,
    author_id: Option<i64>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>,
) -> Result<Vec<TopicWithAuthor>, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new(
        r#"
        SELECT topic.id,
        topic.author_id,
        topic.title,
        topic.description,
        topic.cover_image_path,
        topic.lesson_count,
        topic.total_duration,
        topic.snip_count,
        topic.created_at
    "#,
    );
    query.push(
        r#"
        , author.name AS author_name,
        author.avatar_path AS author_avatar_path,
        author.created_at AS author_created_at,
        author.lesson_count as author_lesson_count
        "#,
    );

    query.push(" FROM topic");

    query.push(" JOIN author ON author.id = topic.author_id AND author.deleted_at IS NULL");

    build_query(&mut query, search, author_id);

    query.push(" ORDER BY");
    match sort {
        Some(sort) if sort == QuerySort::SnipCount => {
            query.push(" topic.snip_count");
        }
        _ => {
            query.push(" topic.created_at");
        }
    }
    match order {
        Some(sort) if sort == QueryOrder::Asc => {
            query.push(" ASC");
        }
        _ => {
            query.push(" DESC");
        }
    }

    query
        .push(" LIMIT ")
        .push_bind(limit as i64)
        .push(" OFFSET ")
        .push_bind(offset as i64)
        .build_query_as::<TopicWithAuthor>()
        .fetch_all(db)
        .await
}

pub async fn count(
    db: &PgPool,
    search: &Option<String>,
    author_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM topic");
    build_query(&mut query, search, author_id);
    Ok(query.build_query_as::<(i64,)>().fetch_one(db).await?.0)
}

pub async fn page_cursor(
    db: &PgPool,
    limit: u32,
    cursor: Option<TopicCursor>,
    search: &Option<String>,
    user_id: Option<i64>,
    author_id: Option<i64>,
    status: &Option<UserProgressStatus>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>,
) -> Result<Vec<TopicWithAuthor>, sqlx::Error> {
    let (order, order_sign) = if order == Some(QueryOrder::Asc) {
        ("ASC", ">")
    } else {
        ("DESC", "<")
    };

    let mut query = QueryBuilder::<Postgres>::new(
        r#"
        SELECT topic.id,
        topic.author_id, 
        topic.title, 
        topic.description, 
        topic.cover_image_path,
        topic.lesson_count,
        topic.total_duration,
        topic.snip_count,
        topic.created_at,
    "#,
    );
    query.push(
        r#"
        author.name AS author_name,
        author.avatar_path AS author_avatar_path, 
        author.created_at AS author_created_at, 
        author.lesson_count as author_lesson_count
        "#,
    );

    if user_id.is_some() {
        query.push(", topic_progress.completed_lesson_count");
    }
    query.push(" FROM topic");

    query.push(" JOIN author ON author.id = topic.author_id AND author.deleted_at IS NULL");

    if let Some(user_id) = user_id {
        query
            .push(" LEFT JOIN topic_progress ON")
            .push(" topic_progress.topic_id = topic.id")
            .push(" AND topic_progress.author_id = topic.author_id")
            .push(" AND topic_progress.user_id = ")
            .push_bind(user_id);
    }

    let has_where = build_query(&mut query, search, author_id);

    if let Some(status) = status {
        if let UserProgressStatus::InProgress = status {
            query.push(if has_where { " AND " } else { " WHERE " })
                .push("topic_progress.completed_lesson_count != topic.lesson_count");
        }
    }

    if let Some(cursor) = cursor {
        query.push(if has_where { " AND " } else { " WHERE " });
        match sort {
            Some(QuerySort::SnipCount) => {
                query
                    .push("(topic.snip_count,topic.id)")
                    .push(order_sign)
                    .push("(")
                    .push_bind(cursor.snip_count)
                    .push(",")
                    .push_bind(cursor.id)
                    .push(")");
            }
            Some(QuerySort::CreatedAt) => {
                query
                    .push("(topic.created_at,topic.id)")
                    .push(order_sign)
                    .push("(")
                    .push_bind(cursor.created_at)
                    .push(",")
                    .push_bind(cursor.id)
                    .push(")");
            }
            _ => {
                query.push("topic.id").push(order_sign).push_bind(cursor.id);
            }
        };
    }

    query.push(" ORDER BY ").push(match sort {
        Some(QuerySort::SnipCount) => {
            format!("topic.snip_count {order}, topic.id {order}")
        }
        Some(QuerySort::CreatedAt) => {
            format!("topic.created_at {order}, topic.id {order}")
        }
        None => {
            format!("topic.id {order}")
        }
    });

    query
        .push(" LIMIT ")
        .push_bind(limit as i64)
        .build_query_as::<TopicWithAuthor>()
        .fetch_all(db)
        .await
}

pub async fn get_by_id(db: &PgPool, id: i64) -> Result<Option<TopicWithAuthor>, sqlx::Error> {
    Ok(sqlx::query_as::<_, TopicWithAuthor>(
        r#"
             SELECT
             topic.id,
             topic.author_id,
             topic.title,
             topic.description,
             topic.cover_image_path,
             topic.lesson_count,
             topic.total_duration,
             topic.snip_count,
             topic.created_at,
             author.name AS author_name,
             author.avatar_path AS author_avatar_path,
             author.created_at AS author_created_at,
             author.lesson_count as author_lesson_count
             FROM topic
             JOIN author ON author.id = topic.author_id AND author.deleted_at IS NULL
             WHERE topic.id = $1
             "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

pub async fn get_by_lesson_id(db: &PgPool, id: i64) -> Result<Option<TopicEntity>, sqlx::Error> {
    Ok(sqlx::query_as::<_, TopicEntity>(
        r#"
             SELECT topic.* FROM topic
             JOIN lesson ON lesson.topic_id = topic.id
             WHERE lesson.id = $1
            "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

pub async fn update_stats(
    connection: &mut PgConnection,
    topic_id: i64,
    lesson_delta: i64,
    duration_delta: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE topic
        SET 
            lesson_count = lesson_count + $1,
            total_duration = total_duration + $2
        WHERE id = $3
        "#,
    )
    .bind(lesson_delta)
    .bind(duration_delta)
    .bind(topic_id)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn increase_snip_count(
    connection: &mut PgConnection,
    topic_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE topic
        SET snip_count = snip_count + 1
        WHERE id = $1
        "#,
    )
    .bind(topic_id)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn update_progress(
    connection: &mut PgConnection,
    user_id: i64,
    author_id: i64,
    topic_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO topic_progress (user_id, author_id, topic_id, completed_lesson_count)
        VALUES (
            $1, $2, $3, (
            SELECT COUNT(id)
            FROM lesson_progress
            WHERE user_id = $1
              AND author_id = $2
              AND topic_id = $3
              AND status = 'completed'
            ))
        ON CONFLICT (user_id, author_id, topic_id) DO UPDATE
            SET completed_lesson_count = EXCLUDED.completed_lesson_count
                "#,
    )
    .bind(user_id)
    .bind(author_id)
    .bind(topic_id)
    .execute(&mut *connection)
    .await?;

    Ok(())
}
