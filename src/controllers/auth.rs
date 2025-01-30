use std::{collections::HashMap, sync::Arc};

use crate::{
    auth::{Claims, DiscordTokenResponse},
    error::Error,
    models::User,
    AppState, ENV,
};

use axum::{extract::State, Json};
use axum_extra::extract::Query;
use chrono::{Duration, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    code: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    id: String,
    token: String,
    expires: u64,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Query(LoginRequest { code }): Query<LoginRequest>,
) -> Result<Json<LoginResponse>, Error> {
    let token_response = get_discord_oauth_token(&state.http, &code).await?;
    let discord_user =
        get_discord_user_from_token(&state.http, &token_response.access_token).await?;

    let id = discord_user.id.to_string();

    sqlx::query!(
        "INSERT INTO users (id) VALUES (?) ON CONFLICT(id) DO NOTHING",
        id
    )
    .execute(&state.db)
    .await?;

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(&state.db)
        .await?;

    //fixme
    let expires_in = Local::now() + Duration::days(30);
    let expires = expires_in.timestamp().try_into().unwrap();

    let token = Claims::new(&user, expires).encode()?;

    let response = LoginResponse {
        id: user.id,
        token,
        expires,
    };

    Ok(Json(response))
}

async fn get_discord_oauth_token(
    http: &reqwest::Client,
    code: &str,
) -> Result<DiscordTokenResponse, Error> {
    let mut form: HashMap<&str, &str> = HashMap::new();
    form.insert("client_id", &ENV.discord_client_id);
    form.insert("client_secret", &ENV.discord_client_secret);
    form.insert("redirect_uri", &ENV.discord_redirect_uri);
    form.insert("grant_type", "authorization_code");
    form.insert("code", &code);

    let req = http
        .post("https://discord.com/api/oauth2/token")
        .form(&form);

    let res = req
        .send()
        .await?
        .error_for_status()?
        .json::<DiscordTokenResponse>()
        .await?;

    Ok(res)
}

async fn get_discord_user_from_token(
    http: &reqwest::Client,
    token: &str,
) -> Result<serenity::model::user::User, Error> {
    let response = http
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json::<serenity::model::user::User>()
        .await?;

    Ok(response)
}
