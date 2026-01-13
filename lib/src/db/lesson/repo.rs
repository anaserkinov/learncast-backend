use crate::db::lesson::entity::{LessonEntity, LessonInput, LessonProgressEntity, LessonWithAuthorTopic};
use sqlx::{Executor, PgConnection, PgPool, Postgres, QueryBuilder};
use time::OffsetDateTime;
use crate::module::common::enums::{UserProgressStatus};
use crate::module::common::lesson::dto::QuerySort;
use crate::module::common::paging::QueryOrder;
use crate::module::user::lesson::dto::LessonCursor;

pub async fn insert(
    connection: &mut PgConnection,
    lesson: LessonInput,
) -> Result<i64, sqlx::Error> {
    let id = sqlx::query_as::<_, (i64,)>(
        r#"
            INSERT INTO lesson (author_id, topic_id, title, description, cover_image_path, audio_path, duration, file_size)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
    )
        .bind(lesson.author_id)
        .bind(lesson.topic_id)
        .bind(lesson.title)
        .bind(lesson.description)
        .bind(lesson.cover_image_path)
        .bind(lesson.audio_path)
        .bind(lesson.duration)
        .bind(lesson.file_size)
        .fetch_one(connection)
        .await?.0;
    Ok(id)
}

pub async fn update(
    connection: &mut PgConnection,
    id: i64,
    lesson: LessonInput,
) -> Result<Option<i64>, sqlx::Error> {
    let id = sqlx::query_as::<_, (i64,)>(
        r#"
        UPDATE lesson
        SET title = $1,
            description = $2,
            cover_image_path = $3,
            audio_path = $4,
            duration = $5,
            file_size = $6
        WHERE id = $7
        RETURNING *
        "#,
    )
    .bind(lesson.title)
    .bind(lesson.description)
    .bind(lesson.cover_image_path)
    .bind(lesson.audio_path)
    .bind(lesson.duration)
    .bind(lesson.file_size)
    .bind(id)
    .fetch_optional(connection)
    .await?.map(|t| t.0 );
    Ok(id)
}

pub async fn delete(
    connection: &mut PgConnection,
    id: i64
) -> Result<Option<LessonEntity>, sqlx::Error> {
    let result = sqlx::query_as::<_, LessonEntity>(
        r#"
               UPDATE lesson SET deleted_at = NOW() WHERE id = $1
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
               SELECT id, deleted_at FROM lesson WHERE deleted_at IS NOT NULL AND deleted_at >= $1
            "#
    )
        .bind(since)
        .fetch_all(db)
        .await?;
    Ok(result)
}

fn build_query(
    query: &mut QueryBuilder<Postgres>,
    author_id: Option<i64>,
    topic_id: Option<i64>,
    search: &Option<String>,
    user_id: Option<i64>,
    status: &Option<UserProgressStatus>,
    favourite: Option<bool>,
) -> bool {
    let mut has_where = false;
    query.push(" JOIN author ON lesson.author_id = author.id");
    query.push(" LEFT JOIN topic ON lesson.topic_id = topic.id");

    if let Some(user_id) = user_id {
        query.push(" LEFT JOIN lesson_progress ON lesson_progress.lesson_id = lesson.id AND lesson_progress.user_id = ")
            .push_bind(user_id);
    }
    if favourite.is_some() {
        query.push(" LEFT JOIN favourite_lesson ON favourite_lesson.lesson_id = lesson.id AND favourite_lesson.user_id = ")
            .push_bind(user_id);
    }

    query.push(" WHERE lesson.deleted_at IS NULL");
    has_where = true;

    if let Some(author_id) = author_id {
        query.push(if has_where { " AND " } else { " WHERE " })
            .push("author_id =").push_bind(author_id);
        has_where = true;
    }

    if let Some(topic_id) = topic_id {
        query
            .push(if has_where { " AND " } else { " WHERE " })
            .push("topic_id = ")
            .push_bind(topic_id);
        has_where = true;
    }

    if let Some(status) = status {
        if let UserProgressStatus::InProgress = status {
            query
                .push(if has_where { " AND " } else { " WHERE " })
                .push("lesson_progress.status = 'in_progress'");
            has_where = true;
        }
    }

    if let Some(is_favourite) = favourite {
        query
            .push(if has_where { " AND " } else { " WHERE " })
            .push("favourite_lesson.lesson_id IS NOT NULL");
        has_where = true;
    }

    if let Some(search) = search {
        query
            .push(if has_where { " AND " } else { " WHERE " })
            .push("lesson.title ILIKE ")
            .push_bind(format!("{}%", search));
        has_where = true;
    }

    has_where
}

pub async fn page(
    db: &PgPool,
    limit: u32,
    offset: u32,
    author_id: Option<i64>,
    topic_id: Option<i64>,
    search: &Option<String>,
    user_id: Option<i64>,
    status: &Option<UserProgressStatus>,
    favourite: Option<bool>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>,
) -> Result<Vec<LessonWithAuthorTopic>, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT");
    query.push(r#"
        lesson.*,

        author.id AS author_id,
        author.name AS author_name,
        author.avatar_path AS author_avatar_path,
        author.created_at AS author_created_at,
        author.lesson_count AS author_lesson_count,

        topic.id AS topic_id,
        topic.title AS topic_title,
        topic.description AS topic_description,
        topic.cover_image_path AS topic_cover_image_path,
        topic.created_at AS topic_created_at,
        topic.lesson_count AS topic_lesson_count,
        topic.total_duration AS topic_total_duration,
        topic.snip_count AS topic_snip_count
    "#);

    if let Some(_) = user_id {
        query.push(", lesson_progress.*, (favourite_lesson.user_id IS NOT NULL) AS is_favourite");
    }
    query.push(" FROM lesson");

    build_query(&mut query, author_id, topic_id, search, user_id, status, favourite);

    query.push(" ORDER BY");
    match sort {
        Some(QuerySort::SnipCount) => {
            query.push(" snip_count");
        }
        _ => {
            query.push(" created_at");
        }
    }
    match order {
        Some(QueryOrder::Desc) => {
            query.push(" DESC");
        }
        _ => {
            query.push(" ASC");
        }
    }

    query
        .push(" LIMIT ")
        .push_bind(limit as i64)
        .push(" OFFSET ")
        .push_bind(offset as i64)
        .build_query_as::<LessonWithAuthorTopic>()
        .fetch_all(db)
        .await
}

pub async fn page_cursor(
    db: &PgPool,
    limit: u32,
    cursor: Option<LessonCursor>,
    author_id: Option<i64>,
    topic_id: Option<i64>,
    search: &Option<String>,
    user_id: Option<i64>,
    status: &Option<UserProgressStatus>,
    favourite: Option<bool>,
    sort: Option<QuerySort>,
    order: Option<QueryOrder>,
) -> Result<Vec<LessonWithAuthorTopic>, sqlx::Error> {
    let (order, order_sign) = if order == Some(QueryOrder::Asc)  { ("ASC", ">") }
    else { ("DESC", "<") };

    let mut query = QueryBuilder::<Postgres>::new("SELECT");
    query.push(r#"
        lesson.*,

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

    if user_id.is_some() {
        query.push(r#"
        , lesson_progress.user_id,
        lesson_progress.started_at,
        lesson_progress.last_position_ms,
        lesson_progress.status,
        lesson_progress.completed_at
        "#);
    }
    if favourite.is_some() {
        query.push(", (favourite_lesson.user_id IS NOT NULL) AS is_favourite");
    }

    query.push(" FROM lesson");

    let has_where = build_query(&mut query, author_id, topic_id, search, user_id, status, favourite);
    if let Some(cursor) = cursor {
        query.push(if has_where { " AND " } else { " WHERE " });
        match sort {
            Some(QuerySort::SnipCount) => {
                query.push("(lesson.snip_count,lesson.id)").push(order_sign)
                    .push("(").push_bind(cursor.snip_count).push(",").push_bind(cursor.id).push(")");
            },
            Some(QuerySort::CreatedAt) => {
                query.push("(lesson.created_at,lesson.id)").push(order_sign)
                    .push("(").push_bind(cursor.created_at).push(",").push_bind(cursor.id).push(")");
            }
            _ => {
                query.push("lesson.id").push(order_sign)
                    .push_bind(cursor.id);
            }
        };
    }

    query.push(" ORDER BY ")
        .push(
            match sort {
                Some(QuerySort::SnipCount) => { format!("lesson.snip_count {order}, lesson.id {order}") },
                Some(QuerySort::CreatedAt) => { format!("lesson.created_at {order}, lesson.id {order}") }
                None => { format!("lesson.id {order}") }
            }
        );

    query
        .push(" LIMIT ")
        .push_bind(limit as i64)
        .build_query_as::<LessonWithAuthorTopic>()
        .fetch_all(db)
        .await
}

pub async fn count(
    db: &PgPool,
    author_id: Option<i64>,
    topic_id: Option<i64>,
    search: &Option<String>,
    user_id: Option<i64>,
    status: &Option<UserProgressStatus>,
    favourite: Option<bool>,
) -> Result<i64, sqlx::Error> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM lesson");
    build_query(&mut query, author_id, topic_id, search, user_id, status, favourite);
    Ok(query.build_query_as::<(i64,)>().fetch_one(db).await?.0)
}

pub async fn get_by_id(db: &PgPool, id: i64) -> Result<Option<LessonEntity>, sqlx::Error> {
    Ok(sqlx::query_as::<_, LessonEntity>(
        r#"
             SELECT * FROM lesson WHERE id = $1
             "#,
    )
        .bind(id)
        .fetch_optional(db)
        .await?)
}

pub async fn get_with_author_topic_by_id(db: &PgPool, id: i64) -> Result<Option<LessonWithAuthorTopic>, sqlx::Error> {
    Ok(sqlx::query_as::<_, LessonWithAuthorTopic>(
        r#"
             SELECT
                 lesson.*,

                 author.id AS author_id,
                 author.name AS author_name,
                 author.avatar_path AS author_avatar_path,
                 author.created_at AS author_created_at,
                 author.lesson_count AS author_lesson_count,

                 topic.id AS topic_id,
                 topic.title AS topic_title,
                 topic.description AS topic_description,
                 topic.cover_image_path AS topic_cover_image_path,
                 topic.created_at AS topic_created_at,
                 topic.lesson_count AS topic_lesson_count,
                 topic.total_duration AS topic_total_duration,
                 topic.snip_count AS topic_snip_count
                 FROM lesson
                 JOIN author ON lesson.author_id = author.id
                 LEFT JOIN topic ON lesson.topic_id = topic.id
                 WHERE lesson.id = $1
             "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

pub async fn get_duration(db: &PgPool, audio_path: String) -> Result<Option<i64>, sqlx::Error> {
    Ok(sqlx::query_as::<_, (i64,)>(
        r#"
             SELECT duration FROM lesson
             WHERE audio_path = $1
             "#,
    )
        .bind(audio_path)
        .fetch_optional(db)
        .await?.map(|o| o.0))
}

pub async fn get_listen_count(db: &PgPool, lesson_id: i64) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_as::<_, (i64,)>(
        r#"
             SELECT listen_count FROM lesson
             WHERE id = $1
             "#,
    )
        .bind(lesson_id)
        .fetch_one(db)
        .await?
        .0)
}

pub async fn create_listen_session(
    connection: &mut PgConnection,
    session_id: String,
    user_id: i64,
    lesson_id: i64,
    created_at: OffsetDateTime
) -> Result<(i64, bool), sqlx::Error> {
    sqlx::query_as::<_, (i64, bool)>(
        r#"
            INSERT INTO listen_session (session_id, user_id, lesson_id, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (session_id) DO UPDATE
                SET session_id = EXCLUDED.session_id
            RETURNING lesson_id, (xmax = 0) AS inserted;
        "#,
    )
        .bind(session_id)
        .bind(user_id)
        .bind(lesson_id)
        .bind(created_at)
        .fetch_one(connection)
        .await
}

pub async fn increase_listen_count(connection: &mut PgConnection, lesson_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE lesson
        SET listen_count = listen_count + 1
        WHERE id = $1
        "#,
    )
    .bind(lesson_id)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn increase_snip_count(
    connection: &mut PgConnection,
    lesson_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE lesson
        SET snip_count = snip_count + 1
        WHERE id = $1
        "#,
    )
    .bind(lesson_id)
    .execute(connection)
    .await?;

    Ok(())
}

pub async fn update_progress(
    connection: &mut PgConnection,
    user_id: i64,
    author_id: i64,
    topic_id: Option<i64>,
    lesson_id: i64,
    started_at: OffsetDateTime,
    last_position_ms: i64,
    status: Option<UserProgressStatus>,
    completed_at: Option<OffsetDateTime>,
) -> Result<LessonProgressEntity, sqlx::Error> {
    let entity = sqlx::query_as::<_, LessonProgressEntity>(
        r#"
        INSERT INTO lesson_progress (user_id, author_id, topic_id, lesson_id, started_at, last_position_ms, status, completed_at)
        VALUES (
            $1, $2, $3, $4, $5, $6, COALESCE($7, 'in_progress'), $8
        )
        ON CONFLICT (user_id, lesson_id) DO UPDATE
            SET last_position_ms = EXCLUDED.last_position_ms,
                status = COALESCE(EXCLUDED.status, lesson_progress.status),
                completed_at = COALESCE(EXCLUDED.completed_at, lesson_progress.completed_at)
        RETURNING *
        "#
    )
        .bind(user_id)
        .bind(author_id)
        .bind(topic_id)
        .bind(lesson_id)
        .bind(started_at)
        .bind(last_position_ms)
        .bind(status)
        .bind(completed_at)
        .fetch_one(connection)
        .await?;
    Ok(entity)
}

pub async fn set_favourite(db: &PgPool, user_id: i64, lesson_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO favourite_lesson (user_id, lesson_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, lesson_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(lesson_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn remove_favourite(
    db: &PgPool,
    user_id: i64,
    lesson_id: i64,
) -> Result<i64, sqlx::Error> {
    let result =
        sqlx::query(r#"DELETE FROM favourite_lesson WHERE user_id = $1 AND lesson_id = $2"#)
            .bind(user_id)
            .bind(lesson_id)
            .execute(db)
            .await?;
    Ok(result.rows_affected() as i64)
}
