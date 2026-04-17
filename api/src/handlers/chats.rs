use crate::{
    AppState,
    dtos::{
        chats::ChatResponse,
        messages::{MessageDto, MessageQuery, SendMessageRequest},
    },
    error::AppError,
    services::chats::ChatService,
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sea_orm::prelude::Uuid;

pub fn chats_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_chats))
        .route("/{chat_id}/messages", get(list_messages).post(send_message))
}

async fn list_chats(State(state): State<AppState>, claims: Claims) -> Json<Vec<ChatResponse>> {
    let response = ChatService::list_chats(&state, claims.sub).await;
    Json(response)
}

async fn list_messages(
    State(state): State<AppState>,
    claims: Claims,
    Path(chat_id): Path<Uuid>,
    Query(params): Query<MessageQuery>,
) -> Json<Vec<MessageDto>> {
    let response = ChatService::list_messages(&state, claims.sub, chat_id, params).await;
    Json(response)
}

async fn send_message(
    State(state): State<AppState>,
    claims: Claims,
    Path(chat_id): Path<Uuid>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<MessageDto, AppError> {
    let response = ChatService::send_message(&state, claims.sub, chat_id, payload.text).await?;
    Ok(response)
}
