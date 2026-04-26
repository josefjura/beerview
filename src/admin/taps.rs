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
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(tap_number): Path<i64>,
    Form(form): Form<SwitchForm>,
) -> impl IntoResponse {
    // 1. Verify CSRF
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    // 2. Verify queue item belongs to this pub
    let queue_item = sqlx::query_as::<_, (i64, i64, Option<String>)>(
        "SELECT id, beer_id, prices FROM queue_item WHERE id = ? AND pub_id = ?"
    )
    .bind(form.queue_item_id)
    .bind(session.pub_id)
    .fetch_optional(&state.db)
    .await;

    let (qi_id, new_beer_id, new_prices) = match queue_item {
        Ok(Some(row)) => row,
        Ok(None) => return (StatusCode::NOT_FOUND, "Queue item not found").into_response(),
        Err(e) => {
            tracing::error!("DB error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // 3. Get current tap position so we can reorder queue later
    let old_position = sqlx::query_as::<_, (i64,)>(
        "SELECT position FROM queue_item WHERE id = ?"
    )
    .bind(qi_id)
    .fetch_one(&state.db)
    .await;

    let old_position = match old_position {
        Ok((pos,)) => pos,
        Err(e) => {
            tracing::error!("DB error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // 4. Run everything in a transaction
    let result = async {
        let mut tx = state.db.begin().await?;

        // Save undo snapshot
        sqlx::query(
            "INSERT OR REPLACE INTO tap_switch_undo (pub_id, tap_number, prev_beer_id, prev_prices, switched_at)
             VALUES (?,
                     ?,
                     (SELECT beer_id FROM tap WHERE pub_id=? AND tap_number=?),
                     (SELECT prices  FROM tap WHERE pub_id=? AND tap_number=?),
                     datetime('now'))"
        )
        .bind(session.pub_id).bind(tap_number)
        .bind(session.pub_id).bind(tap_number)
        .bind(session.pub_id).bind(tap_number)
        .execute(&mut *tx).await?;

        // Archive old tap to history (only if it has a beer)
        sqlx::query(
            "INSERT INTO tap_history (pub_id, tap_number, beer_id, prices, tapped_at, removed_at)
             SELECT pub_id, tap_number, beer_id, prices, updated_at, datetime('now')
             FROM tap WHERE pub_id=? AND tap_number=? AND beer_id IS NOT NULL"
        )
        .bind(session.pub_id).bind(tap_number)
        .execute(&mut *tx).await?;

        // Put new beer on tap
        sqlx::query(
            "UPDATE tap SET beer_id=?, prices=?, updated_at=datetime('now')
             WHERE pub_id=? AND tap_number=?"
        )
        .bind(new_beer_id).bind(&new_prices)
        .bind(session.pub_id).bind(tap_number)
        .execute(&mut *tx).await?;

        // Remove queue item
        sqlx::query("DELETE FROM queue_item WHERE id=?")
            .bind(qi_id)
            .execute(&mut *tx).await?;

        // Reorder remaining queue items
        sqlx::query(
            "UPDATE queue_item SET position = position - 1
             WHERE pub_id=? AND position > ?"
        )
        .bind(session.pub_id).bind(old_position)
        .execute(&mut *tx).await?;

        tx.commit().await?;
        Ok::<_, sqlx::Error>(())
    }.await;

    if let Err(e) = result {
        tracing::error!("switch_tap transaction failed: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Transaction failed").into_response();
    }

    // 5. Fire webhook asynchronously
    let db = state.db.clone();
    let pub_id = session.pub_id;
    tokio::spawn(async move {
        if let Err(e) = crate::webhook::fire_webhook(&db, pub_id).await {
            tracing::warn!("Webhook failed for pub {pub_id}: {e:?}");
        }
    });

    // 6. Return updated tap row as HTMX partial
    // Re-query the tap to get fresh data
    let tap_row = sqlx::query_as::<_, (i64, Option<i64>, Option<String>, Option<String>, Option<String>)>(
        "SELECT t.tap_number, t.beer_id, b.name, b.brewery, t.prices
         FROM tap t LEFT JOIN beer b ON b.id = t.beer_id
         WHERE t.pub_id=? AND t.tap_number=?"
    )
    .bind(session.pub_id).bind(tap_number)
    .fetch_one(&state.db)
    .await;

    match tap_row {
        Ok((tn, beer_id, beer_name, beer_brewery, prices)) => {
            let tap_view = crate::admin::taps::TapView {
                tap_number: tn,
                beer_id,
                beer_name,
                beer_brewery,
                prices,
                can_undo: true, // just switched — undo is available
            };
            let html = crate::templates::admin_taps::render_tap_row(&session, &tap_view);
            axum::response::Html(html.into_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to re-query tap after switch: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load updated tap").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct CsrfForm {
    pub csrf_token: String,
}

pub async fn undo_switch(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(tap_number): Path<i64>,
    Form(form): Form<CsrfForm>,
) -> impl IntoResponse {
    // 1. Verify CSRF
    if form.csrf_token != session.csrf_token {
        return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
    }

    // 2. Load undo snapshot
    let snapshot = sqlx::query_as::<_, (Option<i64>, Option<String>, String)>(
        "SELECT prev_beer_id, prev_prices, switched_at
         FROM tap_switch_undo WHERE pub_id=? AND tap_number=?"
    )
    .bind(session.pub_id).bind(tap_number)
    .fetch_optional(&state.db)
    .await;

    let (prev_beer_id, prev_prices, _switched_at) = match snapshot {
        Ok(Some(row)) => row,
        Ok(None) => return (StatusCode::NOT_FOUND, "No undo available for this tap").into_response(),
        Err(e) => {
            tracing::error!("DB error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // 3. Check window (30 seconds) — query returns nothing if expired
    let still_valid = sqlx::query_as::<_, (i64,)>(
        "SELECT 1 FROM tap_switch_undo
         WHERE pub_id=? AND tap_number=? AND switched_at > datetime('now', '-30 seconds')"
    )
    .bind(session.pub_id).bind(tap_number)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if still_valid.is_none() {
        return (StatusCode::CONFLICT, "Undo window has expired").into_response();
    }

    // 4. Transaction: restore tap, delete undo snapshot, remove last history entry
    let result = async {
        let mut tx = state.db.begin().await?;

        sqlx::query(
            "UPDATE tap SET beer_id=?, prices=?, updated_at=datetime('now')
             WHERE pub_id=? AND tap_number=?"
        )
        .bind(prev_beer_id).bind(&prev_prices)
        .bind(session.pub_id).bind(tap_number)
        .execute(&mut *tx).await?;

        sqlx::query(
            "DELETE FROM tap_switch_undo WHERE pub_id=? AND tap_number=?"
        )
        .bind(session.pub_id).bind(tap_number)
        .execute(&mut *tx).await?;

        sqlx::query(
            "DELETE FROM tap_history WHERE rowid = (
                SELECT rowid FROM tap_history
                WHERE pub_id=? AND tap_number=?
                ORDER BY removed_at DESC LIMIT 1
             )"
        )
        .bind(session.pub_id).bind(tap_number)
        .execute(&mut *tx).await?;

        tx.commit().await?;
        Ok::<_, sqlx::Error>(())
    }.await;

    if let Err(e) = result {
        tracing::error!("undo_switch transaction failed: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Transaction failed").into_response();
    }

    // 5. Return updated tap row
    let tap_row = sqlx::query_as::<_, (i64, Option<i64>, Option<String>, Option<String>, Option<String>)>(
        "SELECT t.tap_number, t.beer_id, b.name, b.brewery, t.prices
         FROM tap t LEFT JOIN beer b ON b.id = t.beer_id
         WHERE t.pub_id=? AND t.tap_number=?"
    )
    .bind(session.pub_id).bind(tap_number)
    .fetch_one(&state.db)
    .await;

    match tap_row {
        Ok((tn, beer_id, beer_name, beer_brewery, prices)) => {
            let tap_view = crate::admin::taps::TapView {
                tap_number: tn, beer_id, beer_name, beer_brewery, prices,
                can_undo: false, // just undone
            };
            let html = crate::templates::admin_taps::render_tap_row(&session, &tap_view);
            axum::response::Html(html.into_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to re-query tap after undo: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load updated tap").into_response()
        }
    }
}

pub async fn mark_empty(
    State(_state): State<AppState>,
    Extension(_session): Extension<Session>,
    Path(_tap_number): Path<i64>,
    Form(_form): Form<CsrfForm>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "TODO: mark tap empty")
}
