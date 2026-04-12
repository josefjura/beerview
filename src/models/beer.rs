use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Beer {
    pub id: i64,
    pub name: String,
    pub brewery: String,
    pub style: Option<String>,
    pub abv: Option<f64>,
    pub untappd_id: Option<String>,
    pub created_at: String,
}
