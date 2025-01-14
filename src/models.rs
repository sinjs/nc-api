use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Theme {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub css: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Ban {
    pub user_id: String,
    pub created_at: chrono::NaiveDateTime,
    pub reason: Option<String>,
    pub expires: Option<chrono::NaiveDateTime>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Badge {
    pub id: i64,
    pub user_id: String,
    pub badge: String,
    pub tooltip: String,
    pub badge_type: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub permissions: i64,
}
