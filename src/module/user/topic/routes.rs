use crate::middleware::auth::user_auth_middleware;
use crate::module::user::topic::controller::{deleted_topics, page_topic};
use crate::state::AppState;
use axum::{middleware, routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/topic", get(page_topic))
        .route("/topic/deleted", get(deleted_topics))
        .layer(middleware::from_fn(user_auth_middleware))
}