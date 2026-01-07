use sqlx::{PgConnection, PgPool};
use crate::db::session::entity::SessionEntity;

pub async fn insert(
    connection: &mut PgConnection,
    session: SessionEntity
) -> Result<SessionEntity, sqlx::Error> {
    sqlx::query_as::<_, SessionEntity>(
        r#"
            INSERT INTO session (user_id, refresh_token_hash, user_agent)
            VALUES ($1, $2, $3)
            RETURNING *
            "#
    )
        .bind(session.user_id)
        .bind(session.refresh_token_hash)
        .bind(session.user_agent)
        .fetch_one(connection)
        .await
}

pub async fn update(db: &PgPool, session: SessionEntity) -> Result<SessionEntity, sqlx::Error> {
    sqlx::query_as::<_, SessionEntity>(
        r#"
        UPDATE session
        SET refresh_token_hash = $1,
            last_used_at = $2
        WHERE id = $3
        RETURNING *
        "#
    )
        .bind(&session.refresh_token_hash)
        .bind(&session.last_used_at)
        .bind(session.id)
        .fetch_one(db)
        .await
}

pub async fn find_by_refresh_token_hash(db: &PgPool, refresh_token_hash: String) -> Result<Option<SessionEntity>, sqlx::Error> {
    sqlx::query_as::<_, SessionEntity>("SELECT * FROM session WHERE refresh_token_hash = $1")
        .bind(refresh_token_hash)
        .fetch_optional(db)
        .await
}

pub async fn delete(db: &PgPool, user_id: i64) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(r#"DELETE FROM session WHERE user_id = $1"#)
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() as i64)
}
