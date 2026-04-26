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
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct MessageQuery {
    pub before: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
}
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub chat_id: Uuid,
    pub id: Uuid,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WsEvent {
    NewMessage {
        recipients: Vec<Uuid>,
        message: MessageDto,
    },
    Typing,
}
