# Message Forwarding

This endpoint uses the namespace `message_forwarding`. Please use this namespace when connecting to
the server like the following:

```js
const socket = io("https://api.example.org/message_forwarding");
```

An example of how to use the API is at [`/examples/message_forwarding.html`](/examples/message_forwarding.html)

## Authentication

You must provide a valid authentication token to connect to this socket, the format is as follows:

| Field | Type   | Description              |
| ----- | ------ | ------------------------ |
| token | string | The authentication token |

### Example

```js
const socket = io("https://api.example.org/message_forwarding", {
  auth: { token: "ey..." },
});
```

## Acknowledgements

Some events return an acknowledgement. These will be returned in the following format:

| Field  | Type    | Description                                   |
| ------ | ------- | --------------------------------------------- |
| status | string  | Either `ok` or `error`                        |
| error  | string? | The description of the error, if there is one |

## Data Structures

### Broadcast Event Type

- `MessageCreate`
- `MessageEdit`
- `MessageDelete`
- `ReactionAdd`
- `ReactionRemove`
- `ChannelUpdate`

### Broadcast Event Object

| Field | Type   | Description                                                                            |
| ----- | ------ | -------------------------------------------------------------------------------------- |
| type  | string | [Type of the event emitted](#broadcast-event-type)                                     |
| event | object | [Event data](https://discord.com/developers/docs/events/gateway-events#receive-events) |

## Client to Server Events

### `create_virtual_channel`

This event creates a new virtual channel and adds you to the internal state room of it.

**Note:** This event returns an [acknowledgement](#acknowledgements)

#### Data Structure

| Field        | Type        | Description                                                                            |
| ------------ | ----------- | -------------------------------------------------------------------------------------- |
| channelId    | snowflake   | The Discord Channel ID to broadcast events from                                        |
| channelData  | object      | [Channel Object](https://discord.com/developers/docs/resources/channel#channel-object) |
| allowedUsers | snowflake[] | Users that are allowed to listen to this virtual channel                               |

### `delete_virtual_channel`

This event deletes a virtual channel. You must be the owner of that channel.

**Note:** This event returns an [acknowledgement](#acknowledgements)

#### Data Structure

| Field     | Type      | Description                                                          |
| --------- | --------- | -------------------------------------------------------------------- |
| channelId | snowflake | The Discord Channel ID to delete the associated virtual channel with |

#### Errors

This event can return the following errors:

- `virtual channel not found`

### `listen_to_channel`

This event can be used to listen to a virtual channel to recieve broadcast events from it. You are
only allowed to listen to a channel if your User ID is specified in the allowed users list of the
channel you are trying to listen to.

After acknowledging, a [`broadcast_event_in_channel`](#broadcast_event_in_channel) event with the
`ChannelUpdate` type will be emitted containg the channel data.

**Note:** This event returns an [acknowledgement](#acknowledgements)

#### Data Structure

| Field     | Type      | Description                                   |
| --------- | --------- | --------------------------------------------- |
| channelId | snowflake | The Discord Channel ID to listen to           |
| ownerId   | snowflake | The owner of the virtual channel to listen to |

#### Errors

This event can return the following errors:

- `virtual channel not found`
- `not allowed to view channel`

## Server to Client Events

### `broadcast_event_in_channel`

This event can be used to send a broadcast event into the specified virtual channel. Only the
owner of a virtual channel can broadcast events to all the listeners of the channel.

**Note:** This event returns an [acknowledgement](#acknowledgements)

#### Data Structure

| Field     | Type      | Description                                                    |
| --------- | --------- | -------------------------------------------------------------- |
| channelId | snowflake | The Discord Channel ID of the channel the event was emitted in |
| event     | object    | [Broadcast Event Object](#broadcast-event-object)              |

#### Errors

This event can return the following errors:

- `virtual channel not found`
- `not allowed to broadcast to channel`
