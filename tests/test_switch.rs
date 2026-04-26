mod common;
use beerview::admin::taps::{execute_switch, execute_undo};

#[tokio::test]
async fn test_switch_tap_atomicity() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;

    let old_beer = common::seed_beer(&pool, "Old Beer", "Old Brewery").await;
    let new_beer = common::seed_beer(&pool, "New Beer", "New Brewery").await;

    common::put_beer_on_tap(&pool, pub_id, 1, old_beer).await;
    let qi_id = common::enqueue_beer(&pool, pub_id, new_beer).await;

    execute_switch(&pool, pub_id, 1, qi_id).await.expect("switch failed");

    // Tap now has new beer
    let (beer_id,): (Option<i64>,) = sqlx::query_as(
        "SELECT beer_id FROM tap WHERE pub_id=? AND tap_number=1"
    )
    .bind(pub_id).fetch_one(&pool).await.unwrap();
    assert_eq!(beer_id, Some(new_beer));

    // Queue is empty
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM queue_item WHERE pub_id=?")
        .bind(pub_id).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);

    // History has old beer
    let (hist_beer_id,): (i64,) = sqlx::query_as(
        "SELECT beer_id FROM tap_history WHERE pub_id=? AND tap_number=1"
    )
    .bind(pub_id).fetch_one(&pool).await.unwrap();
    assert_eq!(hist_beer_id, old_beer);
}

#[tokio::test]
async fn test_switch_invalid_queue_item_returns_error() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;

    let result = execute_switch(&pool, pub_id, 1, 99999).await;
    assert!(result.is_err(), "Should fail with invalid queue item id");

    // Tap unchanged
    let (beer_id,): (Option<i64>,) = sqlx::query_as(
        "SELECT beer_id FROM tap WHERE pub_id=? AND tap_number=1"
    )
    .bind(pub_id).fetch_one(&pool).await.unwrap();
    assert!(beer_id.is_none());
}

#[tokio::test]
async fn test_undo_within_window() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;

    let old_beer = common::seed_beer(&pool, "Old Beer", "X").await;
    let new_beer = common::seed_beer(&pool, "New Beer", "X").await;
    common::put_beer_on_tap(&pool, pub_id, 1, old_beer).await;
    let qi_id = common::enqueue_beer(&pool, pub_id, new_beer).await;

    execute_switch(&pool, pub_id, 1, qi_id).await.unwrap();
    execute_undo(&pool, pub_id, 1).await.expect("undo should succeed within window");

    let (beer_id,): (Option<i64>,) = sqlx::query_as(
        "SELECT beer_id FROM tap WHERE pub_id=? AND tap_number=1"
    )
    .bind(pub_id).fetch_one(&pool).await.unwrap();
    assert_eq!(beer_id, Some(old_beer), "Tap should be restored to old beer");
}

#[tokio::test]
async fn test_undo_after_window_expired() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;

    // Manually insert an expired undo snapshot (61 seconds ago)
    sqlx::query(
        "INSERT INTO tap_switch_undo (pub_id, tap_number, prev_beer_id, prev_prices, switched_at)
         VALUES (?, 1, NULL, NULL, datetime('now', '-61 seconds'))"
    )
    .bind(pub_id).execute(&pool).await.unwrap();

    let result = execute_undo(&pool, pub_id, 1).await;
    assert!(result.is_err(), "Undo should fail after 30-second window");
}
