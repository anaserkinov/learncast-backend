use axum::{Router, routing::post};
use crate::state::AppState;
use super::controller::{login, refresh_token, logout};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh-token", post(refresh_token))
        .route("/auth/logout", post(logout))
}
