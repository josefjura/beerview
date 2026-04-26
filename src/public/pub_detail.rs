use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
    http::StatusCode,
};
use crate::config::AppState;

pub async fn show_pub(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let pub_row = sqlx::query_as::<_, (i64, String, Option<String>)>(
        "SELECT id, name, neighbourhood FROM pub WHERE slug=? AND is_offline=false"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await;

    match pub_row {
        Ok(Some((pub_id, name, neighbourhood))) => {
            // Load taps for HTML display
            let taps = sqlx::query_as::<_, (i64, Option<String>, Option<String>, Option<String>)>(
                "SELECT t.tap_number, b.name, b.brewery, t.prices
                 FROM tap t LEFT JOIN beer b ON b.id = t.beer_id
                 WHERE t.pub_id=? ORDER BY t.tap_number"
            )
            .bind(pub_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

            let markup = crate::templates::pub_page::render_pub_page(&slug, &name, neighbourhood.as_deref(), &taps);
            Html(markup.into_string()).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Html("<h1>Pub not found</h1>".to_string())).into_response(),
        Err(e) => {
            tracing::error!("show_pub: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
