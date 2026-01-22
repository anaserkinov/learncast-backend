use sqlx::{postgres::PgPoolOptions, ConnectOptions, PgPool};

pub async fn create_pool(db_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await
        .expect("Cannot connect to Postgres")
}
