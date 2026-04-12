use axum::{
    Extension,
    Form,
    extract::{Path, State},
    response::IntoResponse,
    http::StatusCode,
};
use serde::Deserialize;

use crate::auth::session::Session;
use crate::config::AppState;

pub async fn show_queue(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: show queue page")
}

#[derive(Deserialize)]
pub struct AddQueueForm {
    pub csrf_token: String,
    pub beer_id: i64,
    pub prices: Option<String>,
}

pub async fn add_to_queue(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Form(_form): Form<AddQueueForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: add to queue")
}

#[derive(Deserialize)]
pub struct CsrfForm {
    pub csrf_token: String,
}

pub async fn remove_from_queue(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Path(_id): Path<i64>,
    Form(_form): Form<CsrfForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: remove from queue")
}

#[derive(Deserialize)]
pub struct UpdatePositionForm {
    pub csrf_token: String,
    pub position: i64,
}

pub async fn update_position(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Path(_id): Path<i64>,
    Form(_form): Form<UpdatePositionForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: update queue position")
}
