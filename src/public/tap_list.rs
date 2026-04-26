use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    http::{StatusCode, header},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::config::AppState;

#[derive(Serialize)]
pub struct PubsResponse {
    pub pubs: Vec<PubSummary>,
}

#[derive(Serialize)]
pub struct PubSummary {
    pub slug: String,
    pub name: String,
    pub neighbourhood: Option<String>,
}

#[derive(Serialize)]
pub struct TapsResponse {
    pub schema_version: &'static str,
    pub pub_name: String,
    pub taps: Vec<TapEntry>,
}

#[derive(Serialize)]
pub struct TapEntry {
    pub tap_number: i64,
    pub beer: Option<BeerEntry>,
    pub prices: Option<Value>,
}

#[derive(Serialize)]
pub struct BeerEntry {
    pub id: i64,
    pub name: String,
    pub brewery: String,
    pub style: Option<String>,
    pub abv: Option<f64>,
    pub untappd_id: Option<String>,
}

pub async fn get_taps_json(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    // Load pub
    let pub_row = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM pub WHERE slug=? AND is_offline=false"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await;

    let (pub_id, pub_name) = match pub_row {
        Ok(Some(row)) => row,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "not found"}))).into_response(),
        Err(e) => {
            tracing::error!("get_taps_json: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "database error"}))).into_response();
        }
    };

    let rows = sqlx::query_as::<_, (i64, Option<i64>, Option<String>, Option<String>, Option<String>, Option<f64>, Option<String>, Option<String>)>(
        "SELECT t.tap_number, b.id, b.name, b.brewery, b.style, b.abv, b.untappd_id, t.prices
         FROM tap t
         LEFT JOIN beer b ON b.id = t.beer_id
         WHERE t.pub_id = ?
         ORDER BY t.tap_number"
    )
    .bind(pub_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let taps = rows.into_iter().map(|(tap_number, beer_id, beer_name, brewery, style, abv, untappd_id, prices_str)| {
        let beer = beer_id.map(|id| BeerEntry {
            id,
            name: beer_name.unwrap_or_default(),
            brewery: brewery.unwrap_or_default(),
            style,
            abv,
            untappd_id,
        });
        let prices: Option<Value> = prices_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        TapEntry { tap_number, beer, prices }
    }).collect();

    Json(TapsResponse {
        schema_version: "1",
        pub_name,
        taps,
    }).into_response()
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
