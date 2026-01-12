use axum::{
    http::Method,
    middleware::Next,
    response::IntoResponse,
    body::Body
};

pub async fn cache_control_middleware(
    req: axum::http::Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let method = req.method().clone();
    let mut response = next.run(req).await;

    if method == Method::GET {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("private, max-age=300"),
        );
    }

    response
}