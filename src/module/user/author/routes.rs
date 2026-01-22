use axum::{Router, routing::get, middleware};
use crate::middleware::auth::user_auth_middleware;
use crate::module::user::author::controller::{deleted_authors, page_author};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/author", get(page_author))
        .route("/author/deleted", get(deleted_authors))
        .layer(middleware::from_fn(user_auth_middleware))
}