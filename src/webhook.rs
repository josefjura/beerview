use sqlx::SqlitePool;
use crate::error::AppError;

#[derive(serde::Serialize)]
struct WebhookPayload {
    schema_version: &'static str,
    event: &'static str,
    pub_name: String,
    taps: Vec<TapPayload>,
}

#[derive(serde::Serialize)]
struct TapPayload {
    tap_number: i64,
    beer: Option<BeerPayload>,
    prices: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct BeerPayload {
    id: i64,
    name: String,
    brewery: String,
    style: Option<String>,
    abv: Option<f64>,
    untappd_id: Option<String>,
}

pub async fn fire_webhook(db: &SqlitePool, pub_id: i64) -> Result<(), AppError> {
    // 1. Load pub
    let pub_row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, webhook_url FROM pub WHERE id=?"
    )
    .bind(pub_id)
    .fetch_optional(db)
    .await
    .map_err(AppError::Database)?;

    let (pub_name, webhook_url) = match pub_row {
        Some(row) => row,
        None => return Ok(()),
    };

    let webhook_url = match webhook_url {
        Some(url) if !url.is_empty() => url,
        _ => return Ok(()), // no webhook configured
    };

    // 2. Load current taps
    let rows = sqlx::query_as::<_, (i64, Option<i64>, Option<String>, Option<String>, Option<String>, Option<f64>, Option<String>, Option<String>)>(
        "SELECT t.tap_number, b.id, b.name, b.brewery, b.style, b.abv, b.untappd_id, t.prices
         FROM tap t
         LEFT JOIN beer b ON b.id = t.beer_id
         WHERE t.pub_id=?
         ORDER BY t.tap_number"
    )
    .bind(pub_id)
    .fetch_all(db)
    .await
    .map_err(AppError::Database)?;

    let taps: Vec<TapPayload> = rows.into_iter().map(|(tap_number, beer_id, beer_name, brewery, style, abv, untappd_id, prices_str)| {
        let beer = beer_id.map(|id| BeerPayload {
            id,
            name: beer_name.unwrap_or_default(),
            brewery: brewery.unwrap_or_default(),
            style,
            abv,
            untappd_id,
        });
        let prices: Option<serde_json::Value> = prices_str.as_deref().and_then(|s| serde_json::from_str(s).ok());
        TapPayload { tap_number, beer, prices }
    }).collect();

    let payload = WebhookPayload {
        schema_version: "1",
        event: "taps_changed",
        pub_name,
        taps,
    };

    // 3. POST with 5-second timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    match client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "beerview-webhook/1.0")
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            tracing::debug!("Webhook delivered to {webhook_url}");
        }
        Ok(resp) => {
            tracing::warn!("Webhook to {webhook_url} returned HTTP {}", resp.status());
        }
        Err(e) => {
            tracing::warn!("Webhook to {webhook_url} failed: {e}");
        }
    }

    Ok(())
}
