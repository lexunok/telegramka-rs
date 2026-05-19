use chrono::{DateTime, Utc};
use macros::IntoDataResponse;
use sea_orm::{FromQueryResult, prelude::Uuid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, FromQueryResult, Deserialize, Clone, IntoDataResponse)]
pub struct MessageDto {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub sender_id: Uuid,
    pub text: String,
    pub device_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct MessageQuery {
    pub before: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
}
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub chat_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub id: Uuid,
    pub text: String,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    NewMessage {
        message: MessageDto,
    },
    PresenceSnapshot {
        user_ids: Vec<Uuid>,
    },
    UserPresence {
        user_id: Uuid,
        online: bool,
    },
    Typing {
        chat_id: Uuid,
        user_id: Uuid,
        typing: bool,
    },
    Read {
        chat_id: Uuid,
        user_id: Uuid,
        read_at: DateTime<Utc>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientEvent {
    Typing { chat_id: Uuid, typing: bool },
    MarkRead { chat_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct WsEnvelope {
    pub recipients: Option<Vec<Uuid>>,
    pub event: WsEvent,
}
