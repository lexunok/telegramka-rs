use crate::{
    AppState,
    dtos::{
        auth::{LoginPayload, PasswordResetPayload, RegisterPayload},
        common::{IdResponse, MessageResponse},
    },
    error::AppError,
    services::{auth::AuthService, profile::ProfileService},
    utils::security::generate_tokens,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{post, put},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use sea_orm::prelude::Uuid;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/registration/{id}", post(registration))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route(
            "/password/verification/{email}",
            post(request_to_update_password),
        )
        .route("/password", put(confirm_and_update_password))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthService::login(&state, payload).await?;

    generate_tokens(
        user.id,
        user.email,
        user.first_name,
        user.last_name,
        user.roles,
    )
}

async fn registration(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = AuthService::register_user(&state, id, payload).await?;

    generate_tokens(
        user.id,
        user.email,
        user.first_name,
        user.last_name,
        user.roles,
    )
}

pub async fn refresh(jar: CookieJar) -> Result<impl IntoResponse, AppError> {
    let claims = AuthService::refresh(jar).await?;

    generate_tokens(
        claims.sub,
        claims.email,
        claims.first_name,
        claims.last_name,
        claims.roles,
    )
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let mut access_cookie = Cookie::from("access_token");
    access_cookie.set_path("/");

    let mut refresh_cookie = Cookie::from("refresh_token");
    refresh_cookie.set_path("/");

    jar.remove(access_cookie).remove(refresh_cookie)
}
async fn request_to_update_password(
    State(state): State<AppState>,
    Path(email): Path<String>,
) -> Result<IdResponse, AppError> {
    let verification_id = ProfileService::request_password_reset(&state, email).await?;

    Ok(IdResponse {
        id: verification_id,
    })
}

async fn confirm_and_update_password(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetPayload>,
) -> Result<MessageResponse, AppError> {
    ProfileService::confirm_password_reset(&state, payload).await?;

    Ok(MessageResponse {
        message: "Успешное обновление пароля".to_string(),
    })
}
