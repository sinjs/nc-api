# Authentication

Most endpoints require authentication. This page will cover the endpoints under the `/auth` namespace.

Authentication is done with OAuth2 and Discord as a provider.

## `POST /auth/login`

### Request Body

| Field | Type   | Description                                                |
| ----- | ------ | ---------------------------------------------------------- |
| code  | string | The Discord OAuth2 Code gained from the authorize endpoint |

### Response Body

| Field   | Type      | Description                                |
| ------- | --------- | ------------------------------------------ |
| id      | snowflake | User ID                                    |
| token   | string    | Authentication Token                       |
| expires | number    | Expiry time as a unix timestamp in seconds |
