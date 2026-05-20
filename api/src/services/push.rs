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
    push_devices,
};
use google_fcm1 as fcm1;
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

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

    async fn build_hub() -> Option<
        fcm1::FirebaseCloudMessaging<
            hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
        >,
    > {
        let path = GLOBAL_CONFIG.fcm_service_account_path.as_ref()?;
        let key = yup_oauth2::read_service_account_key(path).await.ok()?;
        let auth = yup_oauth2::ServiceAccountAuthenticator::builder(key)
            .build()
            .await
            .ok()?;
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()
                        .unwrap()
                        .https_or_http()
                        .enable_http1()
                        .build(),
                );
        Some(fcm1::FirebaseCloudMessaging::new(client, auth))
    }

    pub async fn send_new_message_push(
        state: &AppState,
        message: &MessageModel,
        sender_device_id: Option<&str>,
    ) {
        let Some(project_id) = GLOBAL_CONFIG.fcm_project_id.as_ref() else {
            return;
        };
        let Some(mut hub) = Self::build_hub().await else {
            tracing::error!("fcm hub init failed");
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
            Err(e) => {
                tracing::error!("push recipients query error: {e}");
                return;
            }
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
                    tracing::error!("push devices query error: {e}");
                    continue;
                }
            };

            for d in devices {
                if sender_device_id.is_some_and(|id| id == d.device_id) {
                    continue;
                }

                let req = fcm1::api::SendMessageRequest {
                    message: Some(fcm1::api::Message {
                        token: Some(d.fcm_token.clone()),
                        data: Some(std::collections::HashMap::from([
                            ("chat_id".to_string(), message.chat_id.to_string()),
                            ("user_id".to_string(), message.sender_id.to_string()),
                            ("sender_name".to_string(), sender.name.clone()),
                            ("sender_nickname".to_string(), sender.nickname.clone()),
                            (
                                "avatar_url".to_string(),
                                sender.avatar_url.clone().unwrap_or_default(),
                            ),
                            ("text".to_string(), message.text.clone()),
                        ])),
                        android: Some(fcm1::api::AndroidConfig {
                            collapse_key: Some(message.chat_id.to_string()),
                            priority: Some("high".to_string()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    validate_only: Some(false),
                };

                let parent = format!("projects/{project_id}");
                match hub.projects().messages_send(req, &parent).doit().await {
                    Ok(_) => {}
                    Err(e) => {
                        let err = format!("{e:?}");
                        tracing::error!("fcm send error for device {}: {}", d.id, err);
                        if err.contains("UNREGISTERED") || err.contains("INVALID_ARGUMENT") {
                            let mut model: push_devices::ActiveModel = d.into();
                            model.is_active = Set(false);
                            model.updated_at = Set(Utc::now().into());
                            let _ = model.update(&state.conn).await;
                        }
                    }
                }
            }
        }
    }
}
