use axum::{extract::{State, Path}, response::IntoResponse, http::StatusCode};
use crate::config::AppState;

pub async fn show_pub(
    State(_state): State<AppState>,
    Path(_slug): Path<String>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: show pub detail")
}
