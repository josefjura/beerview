use sqlx::SqlitePool;
use crate::error::AppError;

/// Fire the pub's outbound webhook with the full current tap list.
/// Best-effort: caller spawns this in tokio::spawn and logs warnings on failure.
pub async fn fire_webhook(db: &SqlitePool, pub_id: i64) -> Result<(), AppError> {
    // TODO: implement webhook delivery
    // 1. Load pub webhook_url from DB
    // 2. If None, return Ok(()) — no webhook configured
    // 3. Build JSON payload with full tap list
    // 4. POST to webhook_url with 5-second timeout via reqwest
    let _ = (db, pub_id);
    Ok(())
}
