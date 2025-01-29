use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::{Arc, RwLock},
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildChannel, UserId};

use crate::error::Error;

pub mod auth;
pub mod namespaces;

#[derive(Debug, Serialize, Deserialize)]
pub struct VirtualChannel {
    id: VirtualChannelId,
    channel_data: GuildChannel,
    allowed_user_ids: RwLock<HashSet<UserId>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct VirtualChannelId {
    channel_id: ChannelId,
    owner_id: UserId,
}

impl From<(UserId, ChannelId)> for VirtualChannelId {
    fn from((owner_id, channel_id): (UserId, ChannelId)) -> Self {
        Self {
            owner_id,
            channel_id,
        }
    }
}
impl ToString for VirtualChannelId {
    fn to_string(&self) -> String {
        format!("{};{}", self.owner_id, self.channel_id)
    }
}

impl FromStr for VirtualChannelId {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (owner_id_str, channel_id_str) = s
            .split_once(";")
            .ok_or_else(|| anyhow!("Failed to parse virtual channel id: missing semicolon"))?;

        let owner_id = UserId::from_str(owner_id_str)
            .map_err(|e| anyhow!("Failed to parse owner_id: {}", e))?;
        let channel_id = ChannelId::from_str(channel_id_str)
            .map_err(|e| anyhow!("Failed to parse channel_id: {}", e))?;

        Ok(Self {
            owner_id,
            channel_id,
        })
    }
}

#[derive(Clone, Default)]
pub struct VirtualChannels(Arc<RwLock<HashMap<VirtualChannelId, Arc<VirtualChannel>>>>);

impl VirtualChannels {
    pub fn get(&self, id: &VirtualChannelId) -> Option<Arc<VirtualChannel>> {
        self.0.read().unwrap().get(id).cloned()
    }

    pub fn add(&self, virtual_channel: Arc<VirtualChannel>) {
        self.0
            .write()
            .unwrap()
            .insert(virtual_channel.id.clone(), virtual_channel);
    }

    pub fn remove(&self, id: &VirtualChannelId) {
        self.0.write().unwrap().remove(id);
    }

    pub fn remove_from_user(&self, owner_id: &UserId) {
        let ids_to_remove: Vec<VirtualChannelId> = {
            let channel_map_guard = self.0.read().unwrap();
            channel_map_guard
                .iter()
                .filter_map(|(id, _)| {
                    if &id.owner_id == owner_id {
                        Some(id.clone()) // Clone the key to avoid borrowing issues
                    } else {
                        None
                    }
                })
                .collect()
        };

        let mut channel_map_guard = self.0.write().unwrap();
        for id in ids_to_remove {
            channel_map_guard.remove(&id);
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", content = "error", rename_all = "camelCase")]
pub enum Ack {
    Ok,
    Error(Error),
}
