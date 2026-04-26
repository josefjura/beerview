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

#[derive(Debug)]
pub struct TapView {
    pub tap_number: i64,
    pub beer_id: Option<i64>,
    pub beer_name: Option<String>,
    pub beer_brewery: Option<String>,
    pub prices: Option<String>,
    pub can_undo: bool,
}

pub async fn show_taps(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    // 1. Load taps with beer info
    let rows = sqlx::query_as::<_, (i64, Option<i64>, Option<String>, Option<String>, Option<String>)>(
        "SELECT t.tap_number, t.beer_id, b.name, b.brewery, t.prices
         FROM tap t
         LEFT JOIN beer b ON b.id = t.beer_id
         WHERE t.pub_id = ?
         ORDER BY t.tap_number"
    )
    .bind(session.pub_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // 2. Load undo availability
    let undo_rows = sqlx::query_as::<_, (i64,)>(
        "SELECT tap_number FROM tap_switch_undo
         WHERE pub_id = ? AND switched_at > datetime('now', '-30 seconds')"
    )
    .bind(session.pub_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let undo_set: std::collections::HashSet<i64> = undo_rows.into_iter().map(|(n,)| n).collect();

    let taps: Vec<TapView> = rows.into_iter().map(|(tap_number, beer_id, beer_name, beer_brewery, prices)| {
        TapView {
            tap_number,
            beer_id,
            beer_name,
            beer_brewery,
            prices,
            can_undo: undo_set.contains(&tap_number),
        }
    }).collect();

    // 3. Render
    let markup = crate::templates::admin_taps::render_taps_page(&session, &taps);
    axum::response::Html(markup.into_string())
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
