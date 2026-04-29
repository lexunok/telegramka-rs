use axum::{
    extract::ws::{Message, WebSocket},
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use entity::{chat_members, prelude::*};
use futures_util::{SinkExt, StreamExt};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, prelude::Uuid};
use tokio::sync::broadcast::error::RecvError;

use crate::{
    AppState,
    dtos::messages::{WsClientEvent, WsEnvelope, WsEvent},
    services::chats::ChatService,
    utils::security::Claims,
};

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

    let presence_snapshot = {
        let online_users = state.online_users.read().await;
        online_users.keys().copied().collect::<Vec<_>>()
    };

    if let Ok(json) = serde_json::to_string(&WsEvent::PresenceSnapshot {
        user_ids: presence_snapshot,
    }) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    let became_online = {
        let mut online_users = state.online_users.write().await;
        let count = online_users.entry(user_id).or_insert(0);
        *count += 1;
        *count == 1
    };

    if became_online {
        let _ = state.tx.send(WsEnvelope {
            recipients: None,
            event: WsEvent::UserPresence {
                user_id,
                online: true,
            },
        });
    }

    let send_state = state.clone();
    let recv_state = state.clone();

    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(envelope) => {
                    let should_send = match envelope.recipients.as_ref() {
                        Some(recipients) => recipients.contains(&user_id),
                        None => true,
                    };

                    if !should_send {
                        continue;
                    }

                    let Ok(json) = serde_json::to_string(&envelope.event) else {
                        continue;
                    };

                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => break,
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => {
                    let Ok(event) = serde_json::from_str::<WsClientEvent>(&text) else {
                        continue;
                    };

                    match event {
                        WsClientEvent::Typing { chat_id, typing } => {
                            if let Some(recipients) =
                                load_other_chat_members(&recv_state, chat_id, user_id).await
                            {
                                let _ = recv_state.tx.send(WsEnvelope {
                                    recipients: Some(recipients),
                                    event: WsEvent::Typing {
                                        chat_id,
                                        user_id,
                                        typing,
                                    },
                                });
                            }
                        }
                        WsClientEvent::MarkRead { chat_id } => {
                            let _ = ChatService::mark_read(&recv_state, user_id, chat_id).await;
                        }
                    }
                }
                Message::Close(_) => break,
                Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    let became_offline = {
        let mut online_users = send_state.online_users.write().await;
        match online_users.get_mut(&user_id) {
            Some(count) if *count > 1 => {
                *count -= 1;
                false
            }
            Some(_) => {
                online_users.remove(&user_id);
                true
            }
            None => false,
        }
    };

    if became_offline {
        let _ = send_state.tx.send(WsEnvelope {
            recipients: None,
            event: WsEvent::UserPresence {
                user_id,
                online: false,
            },
        });
    }
}

async fn load_other_chat_members(
    state: &AppState,
    chat_id: Uuid,
    user_id: Uuid,
) -> Option<Vec<Uuid>> {
    let members = ChatMembers::find()
        .filter(chat_members::Column::ChatId.eq(chat_id))
        .select_only()
        .column(chat_members::Column::UserId)
        .into_tuple::<Uuid>()
        .all(&state.conn)
        .await
        .ok()?;

    if !members.contains(&user_id) {
        return None;
    }

    Some(
        members
            .into_iter()
            .filter(|member_id| member_id != &user_id)
            .collect(),
    )
}
