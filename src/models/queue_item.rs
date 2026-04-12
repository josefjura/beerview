use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: i64,
    pub pub_id: i64,
    pub beer_id: i64,
    pub prices: Option<String>,
    pub position: i64,
    pub created_at: String,
}
