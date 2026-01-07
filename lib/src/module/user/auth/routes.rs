use axum::{Router, routing::post};
use crate::state::AppState;
use super::controller::{signin, refresh_token, logout};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signin", post(signin))
        .route("/auth/refresh-token", post(refresh_token))
        .route("/auth/logout", post(logout))
}
