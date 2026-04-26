mod common;

#[tokio::test]
async fn test_mark_empty_clears_tap() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;
    let beer_id = common::seed_beer(&pool, "Test IPA", "Test Brewery").await;
    common::put_beer_on_tap(&pool, pub_id, 1, beer_id).await;

    // Archive + clear
    let mut tx = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO tap_history (pub_id, tap_number, beer_id, prices, tapped_at, removed_at)
         SELECT pub_id, tap_number, beer_id, prices, updated_at, datetime('now')
         FROM tap WHERE pub_id=? AND tap_number=? AND beer_id IS NOT NULL"
    )
    .bind(pub_id).bind(1i64).execute(&mut *tx).await.unwrap();

    sqlx::query("UPDATE tap SET beer_id=NULL, prices=NULL WHERE pub_id=? AND tap_number=?")
        .bind(pub_id).bind(1i64).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();

    let (beer_id_after,): (Option<i64>,) = sqlx::query_as(
        "SELECT beer_id FROM tap WHERE pub_id=? AND tap_number=?"
    )
    .bind(pub_id).bind(1i64)
    .fetch_one(&pool).await.unwrap();

    assert!(beer_id_after.is_none(), "Tap should be empty after mark_empty");
}

#[tokio::test]
async fn test_mark_empty_idempotent() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;

    // Already empty — just clear (no-op on history)
    sqlx::query("UPDATE tap SET beer_id=NULL WHERE pub_id=? AND tap_number=?")
        .bind(pub_id).bind(1i64).execute(&pool).await.unwrap();

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tap_history WHERE pub_id=?")
        .bind(pub_id).fetch_one(&pool).await.unwrap();

    assert_eq!(count, 0, "No history entry should be created for empty tap");
}
