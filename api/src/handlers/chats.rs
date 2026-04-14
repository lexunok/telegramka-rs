use crate::{
    AppState,
    dtos::{
        chats::{CreateChatRequest, ReadChatRequest},
        messages::SendMessageRequest,
    },
    error::AppError,
    services::{chats::ChatService, messages::MessageService},
    utils::security::Claims,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ChatListQuery {
    #[serde(rename = "cursor")]
    _cursor: Option<String>,
    #[serde(rename = "limit")]
    _limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MessageListQuery {
    #[serde(rename = "cursor")]
    _cursor: Option<String>,
    #[serde(rename = "limit")]
    limit: Option<u32>,
}

pub fn chats_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_chats))
        .route("/", post(create_chat))
        .route("/:chat_id", get(get_chat))
        .route("/:chat_id/read", post(mark_read))
        .route("/:chat_id/messages", get(list_messages))
        .route("/:chat_id/messages", post(send_message))
}

async fn list_chats(
    State(state): State<AppState>,
    claims: Claims,
    Query(query): Query<ChatListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let _ = claims;
    let response = ChatService::list_chats(&state, &claims.sub, query._limit).await?;
    Ok(Json(response))
}

async fn create_chat(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = ChatService::open_or_get_chat(&state, &claims.sub, &payload.nickname).await?;
    Ok(Json(response))
}

async fn get_chat(
    State(state): State<AppState>,
    claims: Claims,
    Path(chat_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let response = ChatService::get_chat(&state, &claims.sub, &chat_id).await?;
    Ok(Json(response))
}

async fn mark_read(
    State(state): State<AppState>,
    claims: Claims,
    Path(chat_id): Path<String>,
    Json(payload): Json<ReadChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = ChatService::mark_as_read(
        &state,
        &claims.sub,
        &chat_id,
        payload.read_through_message_id,
    )
    .await?;
    Ok(Json(response))
}

async fn list_messages(
    State(state): State<AppState>,
    claims: Claims,
    Path(chat_id): Path<String>,
    Query(query): Query<MessageListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let response =
        MessageService::list_messages(&state, &claims.sub, &chat_id, query.limit).await?;
    Ok(Json(response))
}

async fn send_message(
    State(state): State<AppState>,
    claims: Claims,
    Path(chat_id): Path<String>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = MessageService::send_message(&state, &claims.sub, &chat_id, payload).await?;
    Ok(Json(response))
}
