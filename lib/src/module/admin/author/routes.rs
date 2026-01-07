use crate::middleware::auth::admin_auth_middleware;
use crate::state::AppState;
use axum::routing::{delete, post, put};
use axum::{middleware, routing::get, Router};
use crate::module::admin::author::controller::{create_author, delete_author, get_author, page_author, update_author};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/author", post(create_author))
        .route("/author/{id}", put(update_author))
        .route("/author/{id}", get(get_author))
        .route("/author/{id}", delete(delete_author))
        .route("/author", get(page_author))
        .layer(middleware::from_fn(admin_auth_middleware))
}