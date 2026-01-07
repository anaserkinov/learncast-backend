use crate::db::user::entity::UserEntity;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

pub async fn insert(   
    connection: &mut PgConnection,
    user: UserEntity
) -> Result<UserEntity, sqlx::Error> {
    sqlx::query_as::<_, UserEntity>(
        r#"
            INSERT INTO users (first_name, last_name, avatar_path, email, telegram_id, telegram_username, google_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
    )
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.avatar_path)
        .bind(user.email)
        .bind(user.telegram_id)
        .bind(user.telegram_username)
        .bind(user.google_id)
        .fetch_one(connection)
        .await
}

pub async fn update(
    connection: &mut PgConnection,
    user: UserEntity
) -> Result<UserEntity, sqlx::Error> {
    sqlx::query_as::<_, UserEntity>(
        r#"
        UPDATE users
        SET first_name = $1,
            last_name = $2,
            avatar_path = $3,
            email = $4,
            telegram_id = $5,
            telegram_username = $6,
            google_id = $7,
            updated_at = $8
        WHERE id = $9
        RETURNING *
        "#
    )
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(&user.avatar_path)
        .bind(&user.email)
        .bind(&user.telegram_id)
        .bind(user.telegram_username)
        .bind(&user.google_id)
        .bind(OffsetDateTime::now_utc())
        .bind(user.id)
        .fetch_one(connection)
        .await
}

pub async fn find_by_id(db: &PgPool, id: i64) -> Result<Option<UserEntity>, sqlx::Error> {
    sqlx::query_as::<_, UserEntity>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_telegram_id(db: &PgPool, telegram_id: i64) -> Result<Option<UserEntity>, sqlx::Error> {
    sqlx::query_as::<_, UserEntity>("SELECT * FROM users WHERE telegram_id = $1")
        .bind(telegram_id)
        .fetch_optional(db)
        .await
}

pub async fn find_by_google_id(db: &PgPool, google_id: &str) -> Result<Option<UserEntity>, sqlx::Error> {
    sqlx::query_as::<_, UserEntity>("SELECT * FROM users WHERE google_id = $1")
        .bind(google_id)
        .fetch_optional(db)
        .await
}
