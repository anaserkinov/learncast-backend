use axum::{Router, routing::get, middleware};
use axum::routing::{delete, patch, post, put};
use crate::middleware::auth::user_auth_middleware;
use crate::module::user::lesson::controller::{page_lesson, increase_listen_count, update_lesson_progress, set_favourite, remove_favourite, deleted_lessons};
use crate::module::user::topic::controller::deleted_topics;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/lesson", get(page_lesson))
        .route("/lesson/deleted", get(deleted_lessons))
        .route("/lesson/{id}/listen", post(increase_listen_count))
        .route("/lesson/{id}/progress", patch(update_lesson_progress))
        .route("/lesson/{id}/favourite", post(set_favourite))
        .route("/lesson/{id}/favourite", delete(remove_favourite))
        .layer(middleware::from_fn(user_auth_middleware))
}