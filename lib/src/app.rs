use crate::api_docs::{AdminApiDoc, UserApiDoc};
use crate::middleware::auth::origin_middleware;
use crate::middleware::cache::cache_control_middleware;
use crate::module::{admin, common, user};
use crate::state::AppState;
use crate::utils::CONFIG;
use axum::http::{HeaderName, HeaderValue, Method};
use axum::{middleware, Router};
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

pub fn build_app(state: AppState) -> Router {
    let admin_json_path = "/api-doc/admin/openapi.json";
    let user_json_path = "/api-doc/user/openapi.json";

    let admin_config = Config::from(format!("/learncast{}", admin_json_path));
    let user_config = Config::from(format!("/learncast{}", user_json_path));

    let admin_router = Router::new()
        .nest("/v1/admin", admin::auth::routes::routes())
        .nest("/v1/admin", admin::author::routes::routes())
        .nest("/v1/admin", admin::topic::routes::routes())
        .nest("/v1/admin", admin::lesson::routes::routes())
        .layer(middleware::from_fn(origin_middleware))
        .merge(
            SwaggerUi::new("/admin/docs")
                .url(admin_json_path, AdminApiDoc::openapi())
                .config(admin_config),
        );

    let user_router = Router::new()
        .nest("/v1/user", user::auth::routes::routes())
        .nest("/v1/user", user::author::routes::routes())
        .nest("/v1/user", user::topic::routes::routes())
        .nest("/v1/user", user::lesson::routes::routes())
        .nest("/v1/user", user::snip::routes::routes())
        .nest("/v1/file", common::file::routes::routes())
        .layer(middleware::from_fn(cache_control_middleware))
        .merge(
            SwaggerUi::new("/user/docs")
                .url(user_json_path, UserApiDoc::openapi())
                .config(user_config),
        );

    let origins = [
        CONFIG.client_origin.parse::<HeaderValue>().unwrap()
    ];
    Router::new()
        .merge(admin_router)
        .merge(user_router)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
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
}
