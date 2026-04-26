use axum::{extract::State, response::IntoResponse, http::StatusCode, Json};
use serde::Serialize;
use crate::config::AppState;

#[derive(Serialize)]
pub struct PubsListResponse {
    pub pubs: Vec<PubItem>,
}

#[derive(Serialize)]
pub struct PubItem {
    pub slug: String,
    pub name: String,
    pub neighbourhood: Option<String>,
}

pub async fn list_pubs(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT slug, name, neighbourhood FROM pub WHERE is_offline=false ORDER BY name"
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let pubs = rows.into_iter().map(|(slug, name, neighbourhood)| PubItem { slug, name, neighbourhood }).collect();
            Json(PubsListResponse { pubs }).into_response()
        }
        Err(e) => {
            tracing::error!("list_pubs: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "database error"}))).into_response()
        }
    }
}

// HTML discovery page — used by issue #12
pub async fn discovery_page(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT slug, name, neighbourhood FROM pub WHERE is_offline=false ORDER BY name"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let markup = crate::templates::discovery::render_discovery_page(&rows);
    axum::response::Html(markup.into_string())
}
