use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateChatRequest {
    pub nickname: String,
}

#[derive(Debug, Serialize)]
pub struct ChatPreview {
    pub id: String,
    pub name: String,
    pub nickname: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub unread: i32,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatListResponse {
    pub items: Vec<ChatPreview>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatDetailResponse {
    pub chat: ChatPreview,
}

#[derive(Debug, Serialize)]
pub struct ChatCreateResponse {
    pub chat: ChatPreview,
    pub created: bool,
}

#[derive(Debug, Serialize)]
pub struct ChatReadResponse {
    pub ok: bool,
    pub unread: i32,
}

#[derive(Debug, Deserialize)]
pub struct ReadChatRequest {
    pub read_through_message_id: Option<String>,
}
