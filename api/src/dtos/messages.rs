use chrono::{DateTime, Utc};
use macros::IntoDataResponse;
use sea_orm::{FromQueryResult, prelude::Uuid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, FromQueryResult, IntoDataResponse)]
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
    pub text: String,
}