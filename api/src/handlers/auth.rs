use crate::{
    AppState,
    dtos::auth::{
        LoginRequest, RefreshRequest, RefreshResponse, RegisterRequest, VerifyCodeRequest,
        VerifyCodeResponse,
    },
    error::AppError,
    services::auth::AuthService,
};
use axum::{Json, Router, extract::State, routing::post};

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/verify-code", post(verify_code))
        .route("/refresh", post(refresh))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(), AppError> {
    AuthService::login(&state, payload.email).await
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(), AppError> {
    AuthService::register(&state, payload).await
}

async fn verify_code(
    State(state): State<AppState>,
    Json(payload): Json<VerifyCodeRequest>,
) -> Result<VerifyCodeResponse, AppError> {
    let response = AuthService::verify_code(&state, payload).await?;
    Ok(response)
}

async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<RefreshResponse, AppError> {
    let response = AuthService::refresh(&state, payload).await?;
    Ok(response)
}
