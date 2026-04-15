use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CodeEmailContext {
    pub code: String,
    pub email: String,
    pub subject: String,
    pub text: String,
}
