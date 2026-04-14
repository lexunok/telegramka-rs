use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CodeEmailContext {
    pub code: String,
    pub email: String,
    pub subject: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct Notification {
    pub email: String,
    pub title: String,
    pub message: String,
    pub link: String,
    pub button_name: String,
}
