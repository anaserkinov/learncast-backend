use axum::{Router, routing::get, middleware};
use axum::routing::{delete, post, put};
use crate::middleware::auth::admin_auth_middleware;
use crate::module::admin::topic::controller::{create_topic, delete_topic, get_topic, page_topic, update_topic};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/topic", post(create_topic))
        .route("/topic/{id}", put(update_topic))
        .route("/topic/{id}", get(get_topic))
        .route("/topic/{id}", delete(delete_topic))
        .route("/topic", get(page_topic))
        .layer(middleware::from_fn(admin_auth_middleware))
}