use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct TapHistory {
    pub id: i64,
    pub pub_id: i64,
    pub tap_number: i64,
    pub beer_id: i64,
    pub prices: Option<String>,
    pub tapped_at: String,
    pub removed_at: String,
}
