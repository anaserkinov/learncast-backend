use axum::{Router, middleware};
use axum::routing::{delete, get, post, put};
use crate::middleware::auth::user_auth_middleware;
use crate::module::user::snip::controller::{count_snip, create_snip, delete_snip, deleted_snips, page_snip, update_snip};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/lesson/{lesson_id}/snip", post(create_snip))
        .route("/lesson/snip/{client_snip_id}", put(update_snip))
        .route("/lesson/snip/{client_snip_id}", delete(delete_snip))
        .route("/lesson/snip", get(page_snip))
        .route("/lesson/snip/deleted", get(deleted_snips))
        .route("/lesson/{lesson_id}/snip/count", get(count_snip))
        .layer(middleware::from_fn(user_auth_middleware))
}