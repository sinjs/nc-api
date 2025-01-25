use std::sync::Arc;

use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef};

use crate::auth::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct SocketAuthData {
    token: String,
}

pub async fn authenticate_middleware(
    socket: SocketRef,
    Data(auth): Data<SocketAuthData>,
) -> Result<(), anyhow::Error> {
    let claims = Arc::new(Claims::decode(&auth.token)?);

    socket.extensions.insert(claims);

    Ok(())
}
