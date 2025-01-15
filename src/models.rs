use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Theme {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub css: String,
}

#[derive(Serialize, Deserialize)]
pub struct Ban {
    pub user_id: String,
    pub created_at: chrono::DateTime<Utc>,
    pub reason: Option<String>,
    pub expires: Option<chrono::DateTime<Utc>>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Ban {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        let created_at: chrono::NaiveDateTime = row.try_get("created_at")?;
        let expires: Option<chrono::NaiveDateTime> = row.try_get("expires")?;

        Ok(Ban {
            user_id: row.try_get("user_id")?,
            created_at: chrono::DateTime::<Utc>::from_naive_utc_and_offset(created_at, Utc),
            reason: row.try_get("reason")?,
            expires: expires.map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)),
        })
    }
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
