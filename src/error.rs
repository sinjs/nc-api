use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::auth::Permissions;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to authenticate")]
    Auth,

    #[error("missing permissions: {missing_permissions:?}")]
    MissingPermissions { missing_permissions: Permissions },

    #[error("not found")]
    NotFound,

    #[error("request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serailizer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serailizer.serialize_str(&self.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(_error: jsonwebtoken::errors::Error) -> Self {
        Self::Auth
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            status: u16,
            message: String,
        }

        let (status, message) = match self {
            Error::Auth => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            Error::NotFound => (StatusCode::NOT_FOUND, "Not Found".into()),
            Error::MissingPermissions { .. } => {
                (StatusCode::FORBIDDEN, "Missing Permissions".into())
            }
            Error::Db(error) => match error.as_database_error() {
                Some(db_error) if db_error.kind() == sqlx::error::ErrorKind::UniqueViolation => (
                    StatusCode::CONFLICT,
                    "Conflict with another resource".into(),
                ),
                _ => match error {
                    sqlx::Error::RowNotFound => {
                        (StatusCode::NOT_FOUND, "Resource Not Found".to_string())
                    }
                    error => {
                        tracing::error!(%error, "database error");

                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Internal Server Error".to_string(),
                        )
                    }
                },
            },
            error => {
                tracing::error!(%error, "internal error");

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
        };

        let error_response = ErrorResponse {
            message,
            status: status.into(),
        };

        (status, Json(error_response)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
