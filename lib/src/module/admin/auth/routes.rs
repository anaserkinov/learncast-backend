use axum::{Router, routing::post, middleware};
use axum::routing::get;
use crate::middleware::auth::admin_auth_middleware;
use crate::state::AppState;
use super::controller::{signin, refresh_token, logout, get_me};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signin", post(signin))
        .route("/auth/refresh-token", post(refresh_token))
        .route("/auth/logout", post(logout))
        .merge(
            Router::new()
                .route("/me", get(get_me))
                .layer(middleware::from_fn(admin_auth_middleware))
        )
}
