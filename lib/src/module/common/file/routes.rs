use axum::{Router, routing::get, middleware};
use axum::extract::DefaultBodyLimit;
use axum::routing::{post};
use crate::middleware::auth::{admin_auth_middleware, common_auth_middleware, origin_middleware};
use crate::module::common::file::controller::{download_file, download_image, upload, upload_url};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    // Public routes (no auth)
    let public = Router::new()
        .route("/image/{*file_path}", get(download_image));

    // Admin-only routes
    let admin = Router::new()
        .route("/", post(upload))
        .layer(DefaultBodyLimit::max(1 * 1024 * 1024))
        .route("/upload-url", get(upload_url))
        .layer(middleware::from_fn(origin_middleware))
        .layer(middleware::from_fn(admin_auth_middleware));

    // Authenticated routes (all users)
    let authenticated = Router::new()
        .route("/{*file_path}", get(download_file))
        .layer(middleware::from_fn(common_auth_middleware));

    Router::new()
        .merge(public)
        .merge(admin)
        .merge(authenticated)
}