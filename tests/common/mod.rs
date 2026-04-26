use sqlx::SqlitePool;

pub async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create in-memory test pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run test migrations");
    pool
}

/// Seed a pub and return its (pub_id, user_id)
pub async fn seed_pub(pool: &SqlitePool) -> (i64, i64) {
    let (pub_id,) = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO pub (slug, name, tap_count) VALUES ('test-pub', 'Test Pub', 4) RETURNING id"
    )
    .fetch_one(pool).await.expect("seed pub");

    // Create taps 1-4
    for tap_num in 1i64..=4 {
        sqlx::query("INSERT INTO tap (pub_id, tap_number) VALUES (?, ?)")
            .bind(pub_id).bind(tap_num)
            .execute(pool).await.expect("seed tap");
    }

    let (user_id,) = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO pub_user (pub_id, username, password_hash) VALUES (?, 'owner', '$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$placeholder') RETURNING id"
    )
    .fetch_one(pool).await.expect("seed user");

    (pub_id, user_id)
}

/// Seed a beer and return its id
pub async fn seed_beer(pool: &SqlitePool, name: &str, brewery: &str) -> i64 {
    let (id,) = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO beer (name, brewery) VALUES (?, ?) RETURNING id"
    )
    .bind(name).bind(brewery)
    .fetch_one(pool).await.expect("seed beer");
    id
}

/// Put a beer on a tap
pub async fn put_beer_on_tap(pool: &SqlitePool, pub_id: i64, tap_number: i64, beer_id: i64) {
    sqlx::query("UPDATE tap SET beer_id=?, updated_at=datetime('now') WHERE pub_id=? AND tap_number=?")
        .bind(beer_id).bind(pub_id).bind(tap_number)
        .execute(pool).await.expect("put beer on tap");
}

/// Add a beer to the queue and return its queue_item id
pub async fn enqueue_beer(pool: &SqlitePool, pub_id: i64, beer_id: i64) -> i64 {
    let (id,) = sqlx::query_as::<_, (i64,)>(
        "INSERT INTO queue_item (pub_id, beer_id, position)
         VALUES (?, ?, (SELECT COALESCE(MAX(position),0)+1 FROM queue_item WHERE pub_id=?))
         RETURNING id"
    )
    .bind(pub_id).bind(beer_id).bind(pub_id)
    .fetch_one(pool).await.expect("enqueue beer");
    id
}
