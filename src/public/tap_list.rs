use axum::{
    extract::{State, Path},
    response::{IntoResponse, Response},
    http::{StatusCode, header},
};
use crate::config::AppState;

pub async fn get_taps_json(
    State(_state): State<AppState>,
    Path(_slug): Path<String>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: get taps JSON")
}

const EMBED_JS: &str = include_str!("../embed/widget.js");

pub async fn serve_embed_js() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(axum::body::Body::from(EMBED_JS))
        .unwrap()
}
