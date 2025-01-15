use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{require_permissions, Claims, Permissions},
    error::Error,
    models::Ban,
    AppState,
};

pub async fn get_ban(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<Ban>, Error> {
    let ban = sqlx::query_as::<_, Ban>("SELECT * FROM bans WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(ban))
}

pub async fn list_bans(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<Vec<Ban>>, Error> {
    require_permissions(claims.permissions(), Permissions::ListBans)?;

    let bans = sqlx::query_as::<_, Ban>("SELECT * FROM bans")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(bans))
}

#[derive(Serialize, Deserialize)]
pub struct CreateBanRequest {
    user_id: String,
    reason: Option<String>,
    expires: Option<DateTime<Utc>>,
}

pub async fn create_ban(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(body): Json<CreateBanRequest>,
) -> Result<Json<Ban>, Error> {
    require_permissions(claims.permissions(), Permissions::ManageBans)?;

    let created_ban = {
        let mut tx = state.db.begin().await?;

        sqlx::query!(
            "INSERT INTO bans (user_id, reason, expires) VALUES (?, ?, ?)",
            body.user_id,
            body.reason,
            body.expires
        )
        .execute(&mut *tx)
        .await?;

        let created_ban = sqlx::query_as::<_, Ban>("SELECT * FROM bans WHERE user_id = ?")
            .bind(body.user_id)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        created_ban
    };

    Ok(Json(created_ban))
}

pub async fn delete_ban(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(user_id): Path<String>,
) -> Result<(), Error> {
    require_permissions(claims.permissions(), Permissions::ManageBans)?;

    let response = sqlx::query!("DELETE FROM bans WHERE user_id = ?", user_id)
        .execute(&state.db)
        .await?;

    if response.rows_affected() != 0 {
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}
