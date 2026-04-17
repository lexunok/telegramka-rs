use crate::dtos::users::UserDto;
use macros::IntoDataResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub nickname: String,
}
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
}


#[derive(Debug, Deserialize)]
pub struct VerifyCodeRequest {
    pub email: String,
    pub code: String,
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct VerifyCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
    pub user: UserDto,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
}
