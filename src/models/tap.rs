use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Tap {
    pub id: i64,
    pub pub_id: i64,
    pub tap_number: i64,
    pub beer_id: Option<i64>,
    pub prices: Option<String>, // JSON: [{"size":"0.5l","price":72}]
    pub updated_at: String,
}
