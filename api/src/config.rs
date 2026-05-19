use jsonwebtoken::{DecodingKey, EncodingKey};
use std::{env, sync::LazyLock};

pub static GLOBAL_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let smtp_from = env::var("SMTP_FROM").expect("SMTP_FROM must be set");
    let smtp_user = env::var("SMTP_USER").unwrap_or_default();
    let smtp_password = env::var("SMTP_PASSWORD").unwrap_or_default();

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let avatar_path = env::var("AVATAR_PATH").expect("AVATAR_PATH must be set");
    let release_path = env::var("RELEASE_PATH").expect("RELEASE_PATH must be set");
    let fcm_server_key = env::var("FCM_SERVER_KEY").ok();
    let fcm_endpoint =
        env::var("FCM_ENDPOINT").unwrap_or("https://fcm.googleapis.com/fcm/send".to_string());

    Config {
        port,
        db_url,
        smtp_host,
        smtp_from,
        smtp_user,
        smtp_password,
        encoding_key,
        decoding_key,
        avatar_path,
        release_path,
        fcm_server_key,
        fcm_endpoint,
    }
});

pub struct Config {
    pub port: String,
    pub db_url: String,
    pub smtp_host: String,
    pub smtp_from: String,
    pub smtp_user: String,
    pub smtp_password: String,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub avatar_path: String,
    pub release_path: String,
    pub fcm_server_key: Option<String>,
    pub fcm_endpoint: String,
}
