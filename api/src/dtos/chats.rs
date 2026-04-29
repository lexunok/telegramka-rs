use chrono::{DateTime, Utc};
use sea_orm::{FromQueryResult, prelude::Uuid};
use serde::Serialize;

#[derive(Debug, Serialize, FromQueryResult)]
pub struct ChatResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub nickname: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub unread: i64,
    pub avatar_url: Option<String>,
}
