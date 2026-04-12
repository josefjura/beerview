mod common;

// TODO: import execute_switch when implemented
// use beerview::admin::taps::execute_switch;

#[tokio::test]
async fn test_switch_tap_atomicity() {
    let _pool = common::test_pool().await;
    // TODO: seed data and test switch atomicity
}

#[tokio::test]
async fn test_switch_invalid_queue_item_returns_error() {
    let _pool = common::test_pool().await;
    // TODO: test that invalid queue_item_id returns error without modifying DB
}

#[tokio::test]
async fn test_undo_within_window() {
    let _pool = common::test_pool().await;
    // TODO: test undo succeeds within 30 seconds
}

#[tokio::test]
async fn test_undo_after_window_expired() {
    let _pool = common::test_pool().await;
    // TODO: test undo rejected after 30 seconds
}
