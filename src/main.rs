mod config;
mod db;
mod state;
mod app;
mod module;
mod utils;
mod api_docs;
mod string_keys;
mod error;
mod extractor;
mod middleware;

use crate::app::build_app;
use crate::state::AppState;
use crate::utils::CONFIG;
use aws_sdk_s3 as s3;
use db::postgres::create_pool;
use redis::Client;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::from_path(std::env::current_dir()?.join(".env")).ok();

    tracing_subscriber::registry()
        .with(
            EnvFilter::new(
                "tower_http=debug,\
             axum=debug,\
             sqlx=debug"
            )
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let db = create_pool(&CONFIG.database_url).await;

    sqlx::migrate!("./src/db/migrations")
        .run(&db)
        .await?;

    println!("✅ Migrations applied");
    
    let redis_client = Client::open(CONFIG.redis_url.clone())
        .expect("Failed to create Redis client");

    let s3_config = aws_config::from_env()
        .endpoint_url(CONFIG.r2_endpoint_url.clone())
        .region("auto")
        .load()
        .await;
    let s3_client = s3::Client::new(&s3_config);

    let state = AppState::new(db, redis_client, s3_client);

    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let local_addr = listener.local_addr()?;
    println!("🚀 Server running on {}", local_addr);

    // use utoipa::openapi::OpenApi;
    // let openapi: OpenApi = AdminApiDoc::openapi();
    // let json = serde_json::to_string_pretty(&openapi).unwrap();
    // std::fs::write("openapi.json", json).unwrap();

    axum::serve(
        listener,
        app
    ).await?;


    Ok(())
}