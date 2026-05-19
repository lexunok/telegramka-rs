use crate::{
    AppState,
    config::GLOBAL_CONFIG,
    dtos::push::{RegisterPushTokenRequest, UnregisterPushTokenRequest},
    error::AppError,
};
use chrono::Utc;
use entity::{
    chat_members,
    messages::Model as MessageModel,
    prelude::{ChatMembers, PushDevices, Users},
    push_devices, users,
};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;

pub struct PushService;

impl PushService {
    pub async fn register_device(
        state: &AppState,
        user_id: Uuid,
        payload: RegisterPushTokenRequest,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        if let Some(existing) = PushDevices::find()
            .filter(push_devices::Column::DeviceId.eq(payload.device_id.clone()))
            .one(&state.conn)
            .await?
        {
            let mut model: push_devices::ActiveModel = existing.into();
            model.user_id = Set(user_id);
            model.platform = Set(payload.platform);
            model.fcm_token = Set(payload.fcm_token);
            model.app_version = Set(payload.app_version);
            model.is_active = Set(true);
            model.last_seen_at = Set(now.into());
            model.updated_at = Set(now.into());
            model.update(&state.conn).await?;
        } else {
            push_devices::ActiveModel {
                user_id: Set(user_id),
                device_id: Set(payload.device_id),
                platform: Set(payload.platform),
                fcm_token: Set(payload.fcm_token),
                app_version: Set(payload.app_version),
                is_active: Set(true),
                last_seen_at: Set(now.into()),
                updated_at: Set(now.into()),
                ..Default::default()
            }
            .insert(&state.conn)
            .await?;
        }
        Ok(())
    }

    pub async fn unregister_device(
        state: &AppState,
        user_id: Uuid,
        payload: UnregisterPushTokenRequest,
    ) -> Result<(), AppError> {
        if let Some(device) = PushDevices::find()
            .filter(push_devices::Column::UserId.eq(user_id))
            .filter(push_devices::Column::DeviceId.eq(payload.device_id))
            .one(&state.conn)
            .await?
        {
            let mut model: push_devices::ActiveModel = device.into();
            model.is_active = Set(false);
            model.updated_at = Set(Utc::now().into());
            model.update(&state.conn).await?;
        }
        Ok(())
    }

    pub async fn send_new_message_push(
        state: &AppState,
        message: &MessageModel,
        sender_device_id: Option<&str>,
    ) {
        let Some(server_key) = GLOBAL_CONFIG.fcm_server_key.as_ref() else {
            return;
        };
        let sender = match Users::find_by_id(message.sender_id).one(&state.conn).await {
            Ok(Some(s)) => s,
            _ => return,
        };
        let recipients = match ChatMembers::find()
            .filter(chat_members::Column::ChatId.eq(message.chat_id))
            .filter(chat_members::Column::UserId.ne(message.sender_id))
            .all(&state.conn)
            .await
        {
            Ok(v) => v,
            Err(_) => return,
        };
        for recipient in recipients {
            let devices = match PushDevices::find()
                .filter(push_devices::Column::UserId.eq(recipient.user_id))
                .filter(push_devices::Column::Platform.eq("android"))
                .filter(push_devices::Column::IsActive.eq(true))
                .all(&state.conn)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("push query error: {e}");
                    continue;
                }
            };
            for d in devices {
                if sender_device_id.is_some_and(|id| id == d.device_id) {
                    continue;
                }
                let payload = json!({"to": d.fcm_token, "collapse_key": message.chat_id.to_string(), "data": {
                    "chat_id": message.chat_id.to_string(),
                    "user_id": message.sender_id.to_string(),
                    "sender_name": sender.name,
                    "sender_nickname": sender.nickname,
                    "avatar_url": sender.avatar_url,
                    "text": message.text,
                }});
                let resp = reqwest::Client::new()
                    .post(&GLOBAL_CONFIG.fcm_endpoint)
                    .bearer_auth(server_key)
                    .json(&payload)
                    .send()
                    .await;
                match resp {
                    Ok(r) if r.status().is_success() => {}
                    Ok(r) => {
                        tracing::error!("fcm push failed status={} device={}", r.status(), d.id);
                        if r.status().as_u16() == 404 || r.status().as_u16() == 410 {
                            let mut m: push_devices::ActiveModel = d.into();
                            m.is_active = Set(false);
                            let _ = m.update(&state.conn).await;
                        }
                    }
                    Err(e) => tracing::error!("fcm send error: {e}"),
                }
            }
        }
    }
}
