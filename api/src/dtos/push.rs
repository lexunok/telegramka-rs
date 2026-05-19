use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterPushTokenRequest {
    pub device_id: String,
    pub platform: String,
    pub fcm_token: String,
    pub app_version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnregisterPushTokenRequest {
    pub device_id: String,
}
