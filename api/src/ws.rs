use axum::{
    extract::ws::{Message, WebSocket},
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use sea_orm::prelude::Uuid;

use crate::{AppState, dtos::messages::WsEvent, utils::security::Claims};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    claims: Claims,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims.sub))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    // --- отправка клиенту
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let WsEvent::NewMessage {
                recipients,
                message,
            } = event
            {
                if recipients.contains(&user_id) {
                    let json = serde_json::to_string(&message).unwrap();

                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // --- приём (пока просто читаем, чтобы соединение не падало)
    let recv_task = tokio::spawn(async move { while let Some(Ok(_)) = receiver.next().await {} });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
