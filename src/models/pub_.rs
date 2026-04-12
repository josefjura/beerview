use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Pub {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub neighbourhood: Option<String>,
    pub tap_count: i64,
    pub webhook_url: Option<String>,
    pub is_offline: bool,
    pub created_at: String,
}
