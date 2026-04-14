use chrono::{DateTime, Utc};
use macros::IntoDataResponse;
use sea_orm::DerivePartialModel;
use serde::Serialize;

#[derive(IntoDataResponse, Debug, Serialize, DerivePartialModel)]
#[sea_orm(entity = "entity::users::Entity")]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}
