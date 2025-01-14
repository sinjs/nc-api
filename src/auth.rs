use std::sync::LazyLock;

use axum::{extract::FromRequestParts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use bitmask_enum::bitmask;
use serde::{Deserialize, Serialize};

use crate::{error::Error, models::User, ENV};

struct Keys {
    encoding: jsonwebtoken::EncodingKey,
    decoding: jsonwebtoken::DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: jsonwebtoken::EncodingKey::from_secret(secret),
            decoding: jsonwebtoken::DecodingKey::from_secret(secret),
        }
    }
}

static KEYS: LazyLock<Keys> = LazyLock::new(|| Keys::new(ENV.jwt_secret.as_bytes()));

#[derive(Serialize, Deserialize)]
pub struct DiscordTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

#[bitmask(i64)]
#[derive(Serialize, Deserialize)]
pub enum Permissions {
    ListBans,
    ManageBans,
    ManageBadges,
    ManageUsers,

    Admin = Self::ListBans.bits | Self::ManageBans.bits | Self::ManageBadges.bits,
    Owner = Self::Admin.bits | Self::ManageUsers.bits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: u64,
    permissions: i64,
}

impl Claims {
    pub fn new(user: &User, exp: u64) -> Self {
        Self {
            exp,
            sub: user.id.to_string(),
            permissions: user.permissions,
        }
    }

    pub fn decode(token: &str) -> Result<Self, crate::Error> {
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &KEYS.decoding,
            &jsonwebtoken::Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub fn encode(&self) -> Result<String, crate::Error> {
        let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &self, &KEYS.encoding)?;

        Ok(token)
    }

    pub fn expires_at(&self) -> u64 {
        self.exp
    }

    pub fn user_id(&self) -> &str {
        &self.sub
    }

    pub fn permissions(&self) -> Permissions {
        tracing::debug!(?self, %self.permissions);
        Permissions::from(self.permissions)
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = crate::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| crate::Error::Auth)?;

        let claims = Claims::decode(bearer.token())?;

        Ok(claims)
    }
}

pub fn require_permissions(
    permissions: Permissions,
    required_permissions: Permissions,
) -> Result<(), Error> {
    tracing::debug!(?permissions, ?required_permissions);
    if permissions.contains(required_permissions) {
        Ok(())
    } else {
        Err(Error::MissingPermissions {
            missing_permissions: required_permissions & !permissions,
        })
    }
}
