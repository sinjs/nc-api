use std::{
    fmt::Debug,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use anyhow::anyhow;
use error::{Error, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

pub mod auth;
pub mod controllers;
pub mod error;
pub mod models;

#[derive(Debug)]
pub struct Env {
    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_redirect_uri: String,
    pub jwt_secret: String,
    pub database_url: String,
    pub database_create: bool,
}

pub static ENV: LazyLock<Env> = LazyLock::new(|| {
    let env = Env {
        discord_client_id: std::env::var("DISCORD_CLIENT_ID")
            .expect("Missing environment variable `DISCORD_CLIENT_ID`"),
        discord_client_secret: std::env::var("DISCORD_CLIENT_SECRET")
            .expect("Missing environment variable `DISCORD_CLIENT_SECRET`"),
        discord_redirect_uri: std::env::var("DISCORD_REDIRECT_URI")
            .expect("Missing environment variable `DISCORD_REDIRECT_URI`"),
        jwt_secret: std::env::var("JWT_SECRET").expect("Missing environment variable `JWT_SECRET`"),
        database_url: std::env::var("DATABASE_URL")
            .expect("Missing environment variable `DATABASE_URL`"),
        database_create: std::env::var("DATABASE_CREATE")
            .unwrap_or("false".to_owned())
            .parse()
            .expect("Invalid boolean value for environment variable `DATABASE_CREATE` (must be `true` or `false`)"),
    };

    tracing::debug!("lazily initialized environment");

    env
});

pub struct AppState {
    pub db: SqlitePool,
    pub http: reqwest::Client,
}

impl AppState {
    pub async fn create() -> Result<Arc<AppState>> {
        let db = SqlitePoolOptions::new()
            .connect_with(
                SqliteConnectOptions::from_str(&ENV.database_url)?
                    .create_if_missing(ENV.database_create),
            )
            .await
            .map_err(|error| Error::Other(anyhow!(error)))?;

        let http = reqwest::ClientBuilder::new()
            .build()
            .map_err(|error| Error::Other(anyhow!(error)))?;

        let state = AppState { db, http };

        Ok(Arc::new(state))
    }
}
