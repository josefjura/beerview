use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct PubUser {
    pub id: i64,
    pub pub_id: i64,
    pub username: String,
    pub password_hash: String,
    pub must_change_password: bool,
    pub created_at: String,
}
