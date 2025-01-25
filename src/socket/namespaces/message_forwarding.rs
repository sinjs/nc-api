use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, RwLock},
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{
        ChannelId, ChannelUpdateEvent, MessageCreateEvent, MessageDeleteEvent, MessageUpdateEvent,
        ReactionAddEvent, ReactionRemoveEvent, UserId,
    },
    json::Value,
};
use socketioxide::extract::{AckSender, Data, Extension, SocketRef, State};

use crate::{
    auth::Claims,
    socket::{Ack, VirtualChannel, VirtualChannelId, VirtualChannels},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CreateVirtualChannelRequest {
    channel_id: ChannelId,
    allowed_user_ids: Vec<UserId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DeleteVirtualChannelRequest {
    channel_id: ChannelId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
enum BroadcastEvent {
    MessageCreate(MessageCreateEvent),
    MessageUpdate(MessageUpdateEvent),
    MessageDelete(MessageDeleteEvent),
    ReactionAdd(ReactionAddEvent),
    ReactionRemove(ReactionRemoveEvent),
    ChannelUpdate(ChannelUpdateEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BroadcastEventInChannel {
    channel_id: ChannelId,
    event: BroadcastEvent,
}

pub fn on_connect(
    socket: SocketRef,
    Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>,
    State(_virtual_channels): State<VirtualChannels>,
) {
    tracing::debug!(%socket.id, ?claims, "socket connected");

    socket.on(
        "create_virtual_channel",
        |socket: SocketRef,
         Data(CreateVirtualChannelRequest {
             channel_id,
             allowed_user_ids,
         }),
         ack: AckSender,
         State::<VirtualChannels>(virtual_channels),
         Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>| async move {
            let allowed_user_ids = RwLock::new(HashSet::from_iter(allowed_user_ids));

            let id = VirtualChannelId {
                channel_id,
                owner_id: claims.user_id().parse().unwrap(),
            };

            let room_id = id.to_string();

            let virtual_channel = Arc::new(VirtualChannel {
                id,
                allowed_user_ids,
            });

            virtual_channels.add(virtual_channel);

            socket.join(room_id);

            ack.send(&Ack::Ok).ok();
        },
    );

    socket.on(
        "delete_virtual_channel",
        |socket: SocketRef,
         ack: AckSender,
         Data(DeleteVirtualChannelRequest { channel_id }),
         State::<VirtualChannels>(virtual_channels),
         Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>| async move {
            let user_id = UserId::from_str(claims.user_id()).unwrap();
            let virtual_channel_id = VirtualChannelId::from((user_id, channel_id));

            if virtual_channels.get(&virtual_channel_id).is_none() {
                ack.send(&Ack::Error(anyhow!("virtual channel not found").into()))
                    .ok();
                return;
            };

            let room_id = virtual_channel_id.to_string();

            socket.leave(room_id);

            virtual_channels.remove(&virtual_channel_id);

            ack.send(&Ack::Ok).ok();
        },
    );

    socket.on(
        "listen_to_channel",
        |socket: SocketRef,
         ack: AckSender,
         Data::<Value>(virtual_channel_id),
         State::<VirtualChannels>(virtual_channels),
         Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>| async move {
            tracing::debug!(?virtual_channel_id, "listen_to_channel");

            let user_id = UserId::from_str(claims.user_id()).unwrap();
            let virtual_channel_id = serenity::json::from_value(virtual_channel_id).unwrap();

            tracing::debug!(%user_id);

            let Some(virtual_channel) = virtual_channels.get(&virtual_channel_id) else {
                ack.send(&Ack::Error(anyhow!("virtual channel not found").into()))
                    .ok();
                return;
            };

            tracing::debug!(%user_id, ?virtual_channel);

            let is_allowed = {
                virtual_channel
                    .allowed_user_ids
                    .read()
                    .unwrap()
                    .iter()
                    .find(|&&id| id == user_id)
                    .is_some()
            };

            if !is_allowed {
                ack.send(&Ack::Error(anyhow!("not allowed to view channel").into()))
                    .ok();
                return;
            }

            socket.join(virtual_channel_id.to_string());

            ack.send(&Ack::Ok).ok();
        },
    );

    socket.on(
        "broadcast_event_in_channel",
        |socket: SocketRef,
         ack: AckSender,
         Data(BroadcastEventInChannel { channel_id, event }),
         State::<VirtualChannels>(virtual_channels),
         Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>| async move {
            let user_id = UserId::from_str(claims.user_id()).unwrap();
            let virtual_channel_id = VirtualChannelId::from((user_id, channel_id));

            let Some(virtual_channel) = virtual_channels.get(&virtual_channel_id) else {
                ack.send(&Ack::Error(anyhow!("virtual channel not found").into()))
                    .ok();
                return;
            };

            if virtual_channel.id.owner_id != user_id {
                ack.send(&Ack::Error(
                    anyhow!("not allowed to broadcast to channel").into(),
                ))
                .ok();
                return;
            }

            socket
                .within(virtual_channel_id.to_string())
                .emit(
                    "broadcast_event_in_channel",
                    &BroadcastEventInChannel { channel_id, event },
                )
                .await
                .ok();

            ack.send(&Ack::Ok).ok();
        },
    );

    socket.on_disconnect(
        |socket: SocketRef,
         State::<VirtualChannels>(virtual_channels),
         Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>| async move {
            tracing::debug!(%socket.id, ?claims, "socket disconnected");

            let owner_id = UserId::from_str(claims.user_id()).unwrap();
            virtual_channels.remove_from_user(&owner_id);
        },
    );
}
