use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::Query;
use serde::{Deserialize, Serialize};

use crate::{
    auth::{require_permissions, Claims, Permissions},
    error::Error,
    models::Badge,
    AppState,
};

pub async fn get_badges_for_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Badge>>, Error> {
    let badges = sqlx::query_as!(Badge, "SELECT * FROM badges WHERE user_id = ?", user_id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(badges))
}

#[derive(Serialize, Deserialize)]
pub struct ListBadgesRequest {
    format: Option<String>,
}

pub async fn list_badges(
    State(state): State<Arc<AppState>>,
    Query(ListBadgesRequest { format }): Query<ListBadgesRequest>,
) -> Result<Response, Error> {
    let badges = sqlx::query_as!(Badge, "SELECT * FROM badges")
        .fetch_all(&state.db)
        .await?;

    if let Some(format) = format {
        if format == "object" {
            let mut object = HashMap::<String, Vec<Badge>>::new();
            for badge in badges {
                object
                    .entry(badge.user_id.clone())
                    .or_insert_with(Vec::new)
                    .push(badge);
            }
            return Ok(Json(object).into_response());
        }
    }

    Ok(Json(badges).into_response())
}

#[derive(Serialize, Deserialize)]
pub struct CreateBadgeRequest {
    user_id: String,
    tooltip: String,
    badge: String,
}

pub async fn create_badge(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Json(body): Json<CreateBadgeRequest>,
) -> Result<(), Error> {
    require_permissions(claims.permissions(), Permissions::ManageBadges)?;

    sqlx::query!(
        "INSERT INTO badges (user_id, tooltip, badge) VALUES (?, ?, ?)",
        body.user_id,
        body.tooltip,
        body.badge
    )
    .execute(&state.db)
    .await?;

    Ok(())
}

pub async fn delete_badge(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(badge_id): Path<String>,
) -> Result<(), Error> {
    require_permissions(claims.permissions(), Permissions::ManageBadges)?;

    let response = sqlx::query!("DELETE FROM badges WHERE id = ?", badge_id)
        .execute(&state.db)
        .await?;

    if response.rows_affected() != 0 {
        Ok(())
    } else {
        Err(Error::NotFound)
    }
}
