use axum::{
    extract::{State, Query},
    response::IntoResponse,
    http::StatusCode,
};
use serde::Deserialize;
use crate::config::AppState;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

pub async fn search_beers(
    State(_state): State<AppState>,
    Query(_params): Query<SearchParams>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: search beers")
}
