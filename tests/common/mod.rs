use sqlx::SqlitePool;

pub async fn test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create test pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run test migrations");
    pool
}
