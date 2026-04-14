# Telegramka Backend API

## Purpose

This document describes the server-side API required to replace the current mock data in the Android client.

Current client flow:

1. User enters email on login screen.
2. If user exists, app opens code verification screen.
3. If user does not exist, app opens registration screen.
4. After registration, app opens code verification screen.
5. After successful code verification, app opens chats list.
6. User can:
   - load chats
   - search chats locally
   - open a chat
   - load messages
   - send messages
   - add a chat by another user's nickname

This spec is intentionally pragmatic: it covers what the current app needs now, plus a small amount of near-term structure so the backend contract does not need to be redesigned immediately.

## General Requirements

- Protocol: HTTPS
- Base path: `/api/v1`
- Format: JSON
- Auth: `Authorization: Bearer <access_token>`
- Time format: Unix timestamp in milliseconds
- IDs: string UUIDs or opaque string IDs
- Pagination: cursor-based where lists can grow

## Error Format

All non-2xx responses should use a common shape:

```json
{
  "error": {
    "code": "USER_NOT_FOUND",
    "message": "User with this email was not found",
    "details": null
  }
}
```

Recommended error codes:

- `VALIDATION_ERROR`
- `UNAUTHORIZED`
- `FORBIDDEN`
- `USER_NOT_FOUND`
- `USER_ALREADY_EXISTS`
- `INVALID_CODE`
- `CODE_EXPIRED`
- `CHAT_NOT_FOUND`
- `MESSAGE_NOT_FOUND`
- `CONTACT_NOT_FOUND`
- `CONTACT_ALREADY_EXISTS`
- `RATE_LIMITED`
- `INTERNAL_ERROR`

## Core Models

### User

```json
{
  "id": "usr_123",
  "name": "Lex",
  "email": "lex@example.com",
  "nickname": "@lex",
  "avatar_url": "https://cdn.example.com/avatar/usr_123.jpg",
  "created_at": 1760000000000
}
```

Fields:

- `id`: unique user id
- `name`: display name
- `email`: unique email
- `nickname`: unique public nickname, stored with leading `@`
- `avatar_url`: nullable
- `created_at`: account creation timestamp

### Auth Session

```json
{
  "access_token": "jwt-or-random-token",
  "refresh_token": "refresh-token",
  "expires_in": 3600,
  "user": {
    "id": "usr_123",
    "name": "Lex",
    "email": "lex@example.com",
    "nickname": "@lex",
    "avatar_url": null,
    "created_at": 1760000000000
  }
}
```

### Chat

Matches the current client model.

```json
{
  "id": "chat_123",
  "name": "Alice",
  "nickname": "@alice",
  "last_message": "Hey, how are you?",
  "last_message_time": 1760000000000,
  "unread": 2,
  "avatar_url": "https://cdn.example.com/avatar/usr_456.jpg"
}
```

Notes:

- Current app treats a chat like a direct dialog preview.
- `name`, `nickname`, and `avatar_url` currently describe the other user in a private chat.
- Backend should still store normalized chat/member data internally even if response is flattened for the client.

### Message

```json
{
  "id": "msg_123",
  "chat_id": "chat_123",
  "sender_id": "usr_123",
  "text": "Hello",
  "timestamp": 1760000000000
}
```

## Authentication Flow

The current app uses email + verification code. No password flow is needed right now.

### 1. Check whether user exists

`POST /auth/check-email`

Request:

```json
{
  "email": "lex@example.com"
}
```

Response:

```json
{
  "exists": true
}
```

Purpose:

- Used on login screen before deciding whether to go to registration or code verification

### 2. Register new user

`POST /auth/register`

Request:

```json
{
  "name": "Lex",
  "email": "lex@example.com",
  "nickname": "@lex"
}
```

Response:

```json
{
  "user": {
    "id": "usr_123",
    "name": "Lex",
    "email": "lex@example.com",
    "nickname": "@lex",
    "avatar_url": null,
    "created_at": 1760000000000
  },
  "verification": {
    "delivery": "email",
    "cooldown_seconds": 60
  }
}
```

Rules:

- `email` must be unique
- `nickname` must be unique
- if nickname is stored with leading `@`, backend should normalize it
- registration should trigger sending a verification code to email

### 3. Send verification code

`POST /auth/send-code`

Request:

```json
{
  "email": "lex@example.com",
  "purpose": "login"
}
```
or
```json
{
  "email": "lex@example.com",
  "purpose": "registration"
}
```

Response:

```json
{
  "sent": true,
  "cooldown_seconds": 60
}
```

Purpose:

- Existing user flow: send code before opening verification screen
- New user flow: may be triggered automatically by `/auth/register`
- Also supports resend later

### 4. Verify code and create session

`POST /auth/verify-code`

Request:

```json
{
  "email": "lex@example.com",
  "code": "123456"
}
```

Response:

```json
{
  "access_token": "jwt-or-random-token",
  "refresh_token": "refresh-token",
  "expires_in": 3600,
  "user": {
    "id": "usr_123",
    "name": "Lex",
    "email": "lex@example.com",
    "nickname": "@lex",
    "avatar_url": null,
    "created_at": 1760000000000
  }
}
```

Purpose:

- Completes both login and registration flow
- App enters authenticated state and opens chats

### 5. Refresh session

`POST /auth/refresh`

Request:

```json
{
  "refresh_token": "refresh-token"
}
```

Response:

```json
{
  "access_token": "new-access-token",
  "refresh_token": "new-refresh-token",
  "expires_in": 3600
}
```

### 6. Logout

`POST /auth/logout`

Headers:

- `Authorization: Bearer <access_token>`

Request:

```json
{
  "refresh_token": "refresh-token"
}
```

Response:

```json
{
  "ok": true
}
```

## Account and User Endpoints

### Get current user

`GET /me`

Response:

```json
{
  "id": "usr_123",
  "name": "Lex",
  "email": "lex@example.com",
  "nickname": "@lex",
  "avatar_url": null,
  "created_at": 1760000000000
}
```

### Update current user

`PATCH /me`

Request:

```json
{
  "name": "Lex",
  "nickname": "@lex"
}
```

Response:

```json
{
  "id": "usr_123",
  "name": "Lex",
  "email": "lex@example.com",
  "nickname": "@lex",
  "avatar_url": null,
  "created_at": 1760000000000
}
```

### Find user by nickname

`GET /users/by-nickname/{nickname}`

Example:

`GET /users/by-nickname/%40alice`

Response:

```json
{
  "id": "usr_456",
  "name": "Alice",
  "email": "alice@example.com",
  "nickname": "@alice",
  "avatar_url": "https://cdn.example.com/avatar/usr_456.jpg",
  "created_at": 1760000000000
}
```

Purpose:

- Needed for adding a new contact/chat by nickname

## Chats

### List chats

`GET /chats`

Query params:

- `cursor` optional
- `limit` optional, default `20`

Response:

```json
{
  "items": [
    {
      "id": "chat_123",
      "name": "Alice",
      "nickname": "@alice",
      "last_message": "Hey, how are you?",
      "last_message_time": 1760000000000,
      "unread": 2,
      "avatar_url": "https://cdn.example.com/avatar/usr_456.jpg"
    }
  ],
  "next_cursor": null
}
```

Purpose:

- Fills chats list screen
- Current search in the app is local; server-side search is optional for now

### Create or open private chat by nickname

`POST /chats`

Request:

```json
{
  "nickname": "@alice"
}
```

Response:

```json
{
  "chat": {
    "id": "chat_123",
    "name": "Alice",
    "nickname": "@alice",
    "last_message": null,
    "last_message_time": null,
    "unread": 0,
    "avatar_url": "https://cdn.example.com/avatar/usr_456.jpg"
  },
  "created": true
}
```

Behavior:

- If private chat already exists, return existing chat with `created: false`
- If target nickname does not exist, return `404 CONTACT_NOT_FOUND`
- If user tries to add themselves, return validation error

### Get chat details

`GET /chats/{chat_id}`

Response:

```json
{
  "id": "chat_123",
  "name": "Alice",
  "nickname": "@alice",
  "last_message": "Hey, how are you?",
  "last_message_time": 1760000000000,
  "unread": 2,
  "avatar_url": "https://cdn.example.com/avatar/usr_456.jpg"
}
```

Purpose:

- Used when chat screen opens and header needs peer info

### Mark chat as read

`POST /chats/{chat_id}/read`

Request:

```json
{
  "read_through_message_id": "msg_999"
}
```

Response:

```json
{
  "ok": true,
  "unread": 0
}
```

Purpose:

- Keeps unread counters accurate

## Messages

### List messages

`GET /chats/{chat_id}/messages`

Query params:

- `cursor` optional
- `limit` optional, default `50`

Response:

```json
{
  "items": [
    {
      "id": "msg_123",
      "chat_id": "chat_123",
      "sender_id": "usr_123",
      "text": "Hello",
      "timestamp": 1760000000000
    }
  ],
  "next_cursor": null
}
```

Notes:

- Client currently renders messages as a flat list
- Results should be returned in chronological order
- Client can reverse locally if needed

### Send message

`POST /chats/{chat_id}/messages`

Request:

```json
{
  "text": "Hello"
}
```

Response:

```json
{
  "message": {
    "id": "msg_124",
    "chat_id": "chat_123",
    "sender_id": "usr_123",
    "text": "Hello",
    "timestamp": 1760000005000
  },
  "chat": {
    "id": "chat_123",
    "name": "Alice",
    "nickname": "@alice",
    "last_message": "Hello",
    "last_message_time": 1760000005000,
    "unread": 0,
    "avatar_url": "https://cdn.example.com/avatar/usr_456.jpg"
  }
}
```

Purpose:

- Adds a message
- Returns updated chat preview so chat list can stay in sync

### Optional future endpoints

Not required for current client, but likely next:

- `PATCH /messages/{message_id}` edit message
- `DELETE /messages/{message_id}` delete message
- attachments upload
- reactions
- typing status
- delivery and read receipts per message

## Suggested Realtime Support

The current client can work with plain REST, but chat will quickly need realtime updates.

Recommended:

- WebSocket: `GET /ws`
- Auth via bearer token during connect

Server events:

- `message.created`
- `message.updated`
- `message.deleted`
- `chat.updated`
- `chat.read`
- `user.presence`

Example event:

```json
{
  "type": "message.created",
  "payload": {
    "id": "msg_124",
    "chat_id": "chat_123",
    "sender_id": "usr_456",
    "text": "Hi",
    "timestamp": 1760000005000
  }
}
```

## Validation Rules

### Email

- required
- valid email format
- case-insensitive uniqueness

### Name

- required
- trimmed
- length 2..64

### Nickname

- required
- unique
- normalized to start with `@`
- allowed chars: latin letters, digits, underscore
- recommended length 3..32 excluding `@`

### Verification Code

- numeric
- 6 digits
- expires, recommended TTL `10 minutes`
- resend cooldown, recommended `60 seconds`
- rate limits per email and IP

### Message Text

- required
- trimmed
- max length, recommended `4000`

## Security and Operational Requirements

- rate limit `/auth/check-email`, `/auth/send-code`, `/auth/verify-code`
- do not leak too much user enumeration data beyond current agreed flow
- store refresh tokens securely and revoke on logout
- audit login and verification attempts
- sanitize all user-generated text
- support idempotency for critical auth/code endpoints if needed

## Minimum Backend Milestone

To support the current app without mocks, the minimum required backend set is:

1. `POST /auth/check-email`
2. `POST /auth/register`
3. `POST /auth/send-code`
4. `POST /auth/verify-code`
5. `GET /me`
6. `GET /chats`
7. `POST /chats`
8. `GET /chats/{chat_id}`
9. `GET /chats/{chat_id}/messages`
10. `POST /chats/{chat_id}/messages`

## Mapping to Current Android Client

Current client models in code:

- `User`: `id`, `name`, `email`, `nickname`
- `Chat`: `id`, `name`, `nickname`, `lastMessage`, `lastMessageTime`, `unread`, `avatarUrl`
- `Message`: `id`, `chatId`, `senderId`, `text`, `timestamp`

Client screens that depend on this API:

- Login screen: email existence check
- Register screen: account creation
- Verify code screen: code confirmation and session start
- Chats screen: fetch chats, add chat by nickname
- Chat screen: fetch chat info, fetch messages, send message

## Open Questions for Backend Implementation

These are not blockers for writing the first version, but should be decided explicitly:

- Will access tokens be JWT or opaque tokens?
- Will chat ids be stable per pair of users for direct chats?
- Should `/auth/check-email` remain explicit, or should auth flow be merged into one start-auth endpoint?
- Will nickname search be exact only, or partial as well?
- Should unread count be per-chat only, or also per-message read state?
- Is message delivery realtime required in first release, or can polling be used initially?
