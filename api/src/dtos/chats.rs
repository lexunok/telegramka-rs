use chrono::{DateTime, Utc};
use sea_orm::{FromQueryResult, prelude::Uuid};
use serde::Serialize;

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ChatResponse {
    pub id: Uuid,
    pub name: String,
    pub nickname: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub unread: i32,
    pub avatar_url: Option<String>,
}
