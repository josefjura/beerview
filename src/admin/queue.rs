use axum::{Extension, Form, extract::{Path, State}, response::IntoResponse, http::StatusCode};
use serde::Deserialize;
use crate::auth::session::Session;
use crate::config::AppState;

#[derive(Debug)]
pub struct QueueItemView {
    pub id: i64,
    pub position: i64,
    pub beer_name: String,
    pub beer_brewery: String,
    pub prices: Option<String>,
}

async fn load_queue(db: &sqlx::SqlitePool, pub_id: i64) -> Result<Vec<QueueItemView>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64, String, String, Option<String>)>(
        "SELECT qi.id, qi.position, b.name, b.brewery, qi.prices
         FROM queue_item qi
         JOIN beer b ON b.id = qi.beer_id
         WHERE qi.pub_id = ?
         ORDER BY qi.position"
    )
    .bind(pub_id)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(|(id, position, beer_name, beer_brewery, prices)| {
        QueueItemView { id, position, beer_name, beer_brewery, prices }
    }).collect())
}

#[derive(Deserialize)]
pub struct AddQueueForm {
    pub csrf_token: String,
    pub beer_id: Option<i64>,
    pub beer_name: Option<String>,
    pub beer_brewery: Option<String>,
    pub beer_style: Option<String>,
    pub beer_abv: Option<f64>,
    pub prices: Option<String>,
}

#[derive(Deserialize)]
pub struct CsrfForm {
    pub csrf_token: String,
}

#[derive(Deserialize)]
pub struct UpdatePositionForm {
    pub csrf_token: String,
    pub position: i64,
}

pub async fn show_queue(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> impl IntoResponse {
    let items = load_queue(&state.db, session.pub_id).await.unwrap_or_default();
    let markup = crate::templates::admin_queue::render_queue_page(&session, &items);
    axum::response::Html(markup.into_string())
}

pub async fn add_to_queue(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Form(form): Form<AddQueueForm>,
) -> impl IntoResponse {
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    let beer_id = if let Some(id) = form.beer_id {
        id
    } else {
        let name = match form.beer_name.as_deref().filter(|s| !s.is_empty()) {
            Some(n) => n.to_string(),
            None => return (StatusCode::UNPROCESSABLE_ENTITY, "Beer name is required").into_response(),
        };
        let brewery = match form.beer_brewery.as_deref().filter(|s| !s.is_empty()) {
            Some(b) => b.to_string(),
            None => return (StatusCode::UNPROCESSABLE_ENTITY, "Brewery is required").into_response(),
        };
        match sqlx::query_as::<_, (i64,)>(
            "INSERT INTO beer (name, brewery, style, abv) VALUES (?, ?, ?, ?) RETURNING id"
        )
        .bind(&name).bind(&brewery).bind(&form.beer_style).bind(form.beer_abv)
        .fetch_one(&state.db).await {
            Ok((id,)) => id,
            Err(e) => {
                tracing::error!("Failed to insert beer: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create beer").into_response();
            }
        }
    };

    if let Err(e) = sqlx::query(
        "INSERT INTO queue_item (pub_id, beer_id, prices, position)
         VALUES (?, ?, ?, (SELECT COALESCE(MAX(position), 0) + 1 FROM queue_item WHERE pub_id=?))"
    )
    .bind(session.pub_id).bind(beer_id).bind(&form.prices).bind(session.pub_id)
    .execute(&state.db).await {
        tracing::error!("Failed to insert queue item: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add to queue").into_response();
    }

    let items = load_queue(&state.db, session.pub_id).await.unwrap_or_default();
    let html = crate::templates::admin_queue::render_queue_list(&session, &items);
    axum::response::Html(html.into_string()).into_response()
}

pub async fn remove_from_queue(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(id): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> impl IntoResponse {
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    let pos = match sqlx::query_as::<_, (i64,)>(
        "SELECT position FROM queue_item WHERE id=? AND pub_id=?"
    )
    .bind(id).bind(session.pub_id)
    .fetch_optional(&state.db).await {
        Ok(Some((p,))) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "Item not found").into_response(),
        Err(e) => { tracing::error!("{e}"); return (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response(); }
    };

    let result = async {
        let mut tx = state.db.begin().await?;
        sqlx::query("DELETE FROM queue_item WHERE id=? AND pub_id=?")
            .bind(id).bind(session.pub_id).execute(&mut *tx).await?;
        sqlx::query("UPDATE queue_item SET position = position - 1 WHERE pub_id=? AND position > ?")
            .bind(session.pub_id).bind(pos).execute(&mut *tx).await?;
        tx.commit().await
    }.await;

    if let Err(e) = result { tracing::error!("remove_from_queue: {e}"); return (StatusCode::INTERNAL_SERVER_ERROR, "Transaction failed").into_response(); }

    let items = load_queue(&state.db, session.pub_id).await.unwrap_or_default();
    let html = crate::templates::admin_queue::render_queue_list(&session, &items);
    axum::response::Html(html.into_string()).into_response()
}

pub async fn update_position(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(id): Path<i64>,
    Form(form): Form<UpdatePositionForm>,
) -> impl IntoResponse {
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    let old_pos = match sqlx::query_as::<_, (i64,)>(
        "SELECT position FROM queue_item WHERE id=? AND pub_id=?"
    )
    .bind(id).bind(session.pub_id)
    .fetch_optional(&state.db).await {
        Ok(Some((p,))) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "Item not found").into_response(),
        Err(e) => { tracing::error!("{e}"); return (StatusCode::INTERNAL_SERVER_ERROR, "DB error").into_response(); }
    };

    let count = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM queue_item WHERE pub_id=?")
        .bind(session.pub_id).fetch_one(&state.db).await.map(|(c,)| c).unwrap_or(1);

    let new_pos = form.position.clamp(1, count);
    if new_pos != old_pos {
        let result = async {
            let mut tx = state.db.begin().await?;
            if new_pos < old_pos {
                sqlx::query("UPDATE queue_item SET position = position + 1 WHERE pub_id=? AND position >= ? AND position < ? AND id != ?")
                    .bind(session.pub_id).bind(new_pos).bind(old_pos).bind(id).execute(&mut *tx).await?;
            } else {
                sqlx::query("UPDATE queue_item SET position = position - 1 WHERE pub_id=? AND position > ? AND position <= ? AND id != ?")
                    .bind(session.pub_id).bind(old_pos).bind(new_pos).bind(id).execute(&mut *tx).await?;
            }
            sqlx::query("UPDATE queue_item SET position=? WHERE id=? AND pub_id=?")
                .bind(new_pos).bind(id).bind(session.pub_id).execute(&mut *tx).await?;
            tx.commit().await
        }.await;
        if let Err(e) = result { tracing::error!("update_position: {e}"); return (StatusCode::INTERNAL_SERVER_ERROR, "Transaction failed").into_response(); }
    }

    let items = load_queue(&state.db, session.pub_id).await.unwrap_or_default();
    let html = crate::templates::admin_queue::render_queue_list(&session, &items);
    axum::response::Html(html.into_string()).into_response()
}
