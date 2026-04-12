use axum::{extract::State, response::IntoResponse, http::StatusCode};
use crate::config::AppState;

pub async fn list_pubs(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: list pubs")
}
