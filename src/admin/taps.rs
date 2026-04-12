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

pub async fn show_taps(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: show taps page")
}

#[derive(Deserialize)]
pub struct SwitchForm {
    pub csrf_token: String,
    pub queue_item_id: i64,
}

pub async fn switch_tap(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Path(_tap_number): Path<i64>,
    Form(_form): Form<SwitchForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: switch tap")
}

#[derive(Deserialize)]
pub struct CsrfForm {
    pub csrf_token: String,
}

pub async fn undo_switch(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Path(_tap_number): Path<i64>,
    Form(_form): Form<CsrfForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: undo switch")
}

pub async fn mark_empty(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Path(_tap_number): Path<i64>,
    Form(_form): Form<CsrfForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: mark tap empty")
}
