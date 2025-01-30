use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, RwLock},
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{
        Channel, ChannelCreateEvent, ChannelId, ChannelUpdateEvent, GuildChannel,
        MessageCreateEvent, MessageDeleteEvent, MessageUpdateEvent, ReactionAddEvent,
        ReactionRemoveEvent, UserId,
    },
    json::{self, Value},
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
    channel_data: GuildChannel,
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
    // fixme not make it value :3
    MessageCreate(Value),
    MessageUpdate(Value),
    MessageDelete(Value),
    ReactionAdd(Value),
    ReactionRemove(Value),
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
    let span = tracing::debug_span!("socket", %socket.id, %claims.sub);
    let _enter = span.enter();

    tracing::debug!("socket connected");

    socket.on(
        "create_virtual_channel",
        |socket: SocketRef,
         Data(CreateVirtualChannelRequest {
             channel_id,
             channel_data,
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
                channel_data,
                allowed_user_ids,
            });

            tracing::debug!(?virtual_channel, "create_virtual_channel");
            tracing::debug!("before {:?}", socket.rooms());

            virtual_channels.add(virtual_channel);

            socket.join(room_id);
            tracing::debug!("after {:?}", socket.rooms());

            ack.send(&Ack::Ok).ok();

            tracing::debug!("final (create_virtual_channel)")
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

            tracing::debug!("before {:?}", socket.rooms());

            socket.join(virtual_channel_id.to_string());

            tracing::debug!("after {:?}", socket.rooms());

            ack.send(&Ack::Ok).ok();

            let Ok(channel_update_event) = json::to_value(virtual_channel.channel_data.clone())
                .and_then(|v| json::from_value::<ChannelUpdateEvent>(v))
            else {
                return;
            };

            tracing::debug!(?channel_update_event);

            socket
                .emit(
                    "broadcast_event_in_channel",
                    &BroadcastEventInChannel {
                        channel_id: virtual_channel_id.channel_id,
                        event: BroadcastEvent::ChannelUpdate(channel_update_event),
                    },
                )
                .ok();
        },
    );

    socket.on(
        "broadcast_event_in_channel",
        |socket: SocketRef,
         ack: AckSender,
         Data(BroadcastEventInChannel { channel_id, event }),
         State::<VirtualChannels>(virtual_channels),
         Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>| async move {
            tracing::debug!(%channel_id, ?event, "broadcast_event_in_channel");
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
        |_socket: SocketRef,
         State::<VirtualChannels>(virtual_channels),
         Extension::<Arc<Claims>>(claims): Extension<Arc<Claims>>| async move {
            tracing::debug!("disconnected");

            let owner_id = UserId::from_str(claims.user_id()).unwrap();
            virtual_channels.remove_from_user(&owner_id);
        },
    );
}
