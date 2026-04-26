mod common;

#[tokio::test]
async fn test_add_to_queue_assigns_correct_position() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;

    let b1 = common::seed_beer(&pool, "Beer A", "Brewery X").await;
    let b2 = common::seed_beer(&pool, "Beer B", "Brewery X").await;
    let b3 = common::seed_beer(&pool, "Beer C", "Brewery X").await;

    common::enqueue_beer(&pool, pub_id, b1).await;
    common::enqueue_beer(&pool, pub_id, b2).await;
    common::enqueue_beer(&pool, pub_id, b3).await;

    let positions: Vec<(i64,)> = sqlx::query_as(
        "SELECT position FROM queue_item WHERE pub_id=? ORDER BY position"
    )
    .bind(pub_id).fetch_all(&pool).await.unwrap();

    assert_eq!(positions, vec![(1,), (2,), (3,)]);
}

#[tokio::test]
async fn test_remove_from_queue_reorders() {
    let pool = common::test_pool().await;
    let (pub_id, _) = common::seed_pub(&pool).await;

    let b1 = common::seed_beer(&pool, "Beer A", "X").await;
    let b2 = common::seed_beer(&pool, "Beer B", "X").await;
    let b3 = common::seed_beer(&pool, "Beer C", "X").await;

    common::enqueue_beer(&pool, pub_id, b1).await;
    let qi2 = common::enqueue_beer(&pool, pub_id, b2).await;
    common::enqueue_beer(&pool, pub_id, b3).await;

    // Remove item at position 2
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("DELETE FROM queue_item WHERE id=? AND pub_id=?")
        .bind(qi2).bind(pub_id).execute(&mut *tx).await.unwrap();
    sqlx::query("UPDATE queue_item SET position=position-1 WHERE pub_id=? AND position>2")
        .bind(pub_id).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();

    let positions: Vec<(i64,)> = sqlx::query_as(
        "SELECT position FROM queue_item WHERE pub_id=? ORDER BY position"
    )
    .bind(pub_id).fetch_all(&pool).await.unwrap();

    assert_eq!(positions, vec![(1,), (2,)], "Remaining items should be at positions 1 and 2");
}
