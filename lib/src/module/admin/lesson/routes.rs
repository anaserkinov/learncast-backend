use axum::{Router, routing::get, middleware};
use axum::routing::{delete, post, put};
use crate::middleware::auth::admin_auth_middleware;
use crate::module::admin::lesson::controller::{create_lesson, delete_lesson, get_lesson, page_lesson, update_lesson};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/lesson", post(create_lesson))
        .route("/lesson/{id}", put(update_lesson))
        .route("/lesson/{id}", get(get_lesson))
        .route("/lesson/{id}", delete(delete_lesson))
        .route("/lesson", get(page_lesson))
        .layer(middleware::from_fn(admin_auth_middleware))
}