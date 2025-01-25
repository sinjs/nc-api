# API Documentation

## Versioning

All API endpoints have a version. The current version is `2`, so it must be specified as follows:

```
/v2/auth/login
```

## Errors

Errors are indicated by their unsuccessful status code, for example `403`. They will return a response in either one of the following formats:

- `text/plain` with the error message as the body
- `application/json` with the following format:

  | Field   | Type   | Description                  |
  | ------- | ------ | ---------------------------- |
  | status  | number | The status code of the error |
  | message | string | The message of the error     |

## Authentication

Some endpoints require authentication. In the documentation, these will be marked with a `🔒` symbol.
The bearer token should be specified in the `Authorization` header:

```http
Authorization: Bearer <token>
```

You may obtain a bearer token from the [`/auth/login`](auth.md#login) endpoint.

## Authorization

Some endpoints require a specific permission level to access. These are specified in the
authentication token under the `permission` field. They are in a bitfield format:

```js
const Permissions = {
  ListBans     = 1 << 0, // 1
  ManageBans   = 1 << 1, // 2
  ManageBadges = 1 << 2, // 4
  ManageUsers  = 1 << 3, // 8
}
```

These endpoints will be marked with a `🛂` symbol with the required permission.
