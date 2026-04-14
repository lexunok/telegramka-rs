use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct MessageDto {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub items: Vec<MessageDto>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub message: MessageDto,
    pub chat: crate::dtos::chats::ChatPreview,
}
