use axum::{
    Extension,
    extract::{State, Query},
    response::{Html, IntoResponse},
};
use maud::html;
use serde::Deserialize;
use crate::auth::session::Session;
use crate::config::AppState;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

pub async fn search_beers(
    State(state): State<AppState>,
    Extension(_session): Extension<Session>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    if params.q.len() < 2 {
        return Html(html! { ul {} }.into_string());
    }

    let pattern = format!("%{}%", params.q);
    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<f64>)>(
        "SELECT id, name, brewery, style, abv FROM beer
         WHERE name LIKE ? OR brewery LIKE ?
         ORDER BY name LIMIT 10"
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let markup = html! {
        ul class="beer-search-results" {
            @for (id, name, brewery, style, abv) in &rows {
                li {
                    button type="button"
                        onclick=(format!(
                            "document.getElementById('selected-beer-id').value='{}'; \
                             document.getElementById('beer-search').value={:?}; \
                             document.getElementById('beer-search-results').innerHTML='';",
                            id, name
                        )) {
                        strong { (name) }
                        " — " (brewery)
                        @if let Some(ref s) = style { " · " (s) }
                        @if let Some(a) = abv { " · " (format!("{:.1}%", a)) }
                    }
                }
            }
            @if rows.is_empty() {
                li class="no-results" { "No beers found — enter details above to add new." }
            }
        }
    };

    Html(markup.into_string())
}
