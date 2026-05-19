use crate::{
    AppState,
    dtos::push::{RegisterPushTokenRequest, UnregisterPushTokenRequest},
    error::AppError,
    services::push::PushService,
    utils::security::Claims,
};
use axum::{Json, Router, extract::State, routing::post};

pub fn push_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/unregister", post(unregister))
}

async fn register(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<RegisterPushTokenRequest>,
) -> Result<(), AppError> {
    PushService::register_device(&state, claims.sub, payload).await
}

async fn unregister(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UnregisterPushTokenRequest>,
) -> Result<(), AppError> {
    PushService::unregister_device(&state, claims.sub, payload).await
}
