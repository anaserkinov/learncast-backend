use crate::api_docs::{AdminApiDoc, UserApiDoc};
use crate::middleware::auth::origin_middleware;
use crate::module::{admin, common, user};
use crate::state::AppState;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::{middleware, Router};
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .nest("/v1/user", user::auth::routes::routes())
        .nest("/v1/user", user::author::routes::routes())
        .nest("/v1/user", user::topic::routes::routes())
        .nest("/v1/user", user::lesson::routes::routes())
        .nest("/v1/user", user::snip::routes::routes())
        .nest("/v1/admin", admin::auth::routes::routes())
        .nest("/v1/admin", admin::author::routes::routes())
        .nest("/v1/admin", admin::topic::routes::routes())
        .nest("/v1/admin", admin::lesson::routes::routes())
        .layer(middleware::from_fn(origin_middleware))
        .nest("/v1/file", common::file::routes::routes())
        .merge(
            SwaggerUi::new("/user/docs").url("/api-doc/user/openapi.json", UserApiDoc::openapi()),
        )
        .merge(
            SwaggerUi::new("/admin/docs")
                .url("/api-doc/admin/openapi.json", AdminApiDoc::openapi()),
        )
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin("http://127.0.0.1".parse::<HeaderValue>().unwrap())
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers(AllowHeaders::list([
                    HeaderName::from_static("authorization"),
                    HeaderName::from_static("content-type"),
                    HeaderName::from_static("accept"),
                ]))
                .allow_credentials(true),
        )
        .with_state(state)
}
