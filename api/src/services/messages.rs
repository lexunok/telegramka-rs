use crate::{
    AppState,
    dtos::messages::{MessageDto, MessageListResponse, SendMessageRequest, SendMessageResponse},
    error::AppError,
    services::chats::ChatService,
};
use chrono::Utc;
use entity::{
    chat_members::{self, Entity as ChatMember},
    chats::Entity as Chat,
    messages::{self, ActiveModel as MessageModel, Entity as Message},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect,
};
use uuid::Uuid;

pub struct MessageService;

impl MessageService {
    pub async fn list_messages(
        state: &AppState,
        user_id: &str,
        chat_id: &str,
        limit: Option<u32>,
    ) -> Result<MessageListResponse, AppError> {
        Self::ensure_membership(state.conn(), user_id, chat_id).await?;

        let limit = limit.unwrap_or(50).clamp(1, 200);
        let messages: Vec<messages::Model> = Message::find()
            .filter(messages::Column::ChatId.eq(chat_id.to_owned()))
            .order_by_asc(messages::Column::CreatedAt)
            .limit(limit as u64)
            .all(state.conn())
            .await?;

        let items = messages
            .into_iter()
            .map(|message| Self::map_message(&message))
            .collect();
        Ok(MessageListResponse {
            items,
            next_cursor: None,
        })
    }

    pub async fn send_message(
        state: &AppState,
        user_id: &str,
        chat_id: &str,
        payload: SendMessageRequest,
    ) -> Result<SendMessageResponse, AppError> {
        let trimmed = payload.text.trim();
        if trimmed.is_empty() || trimmed.len() > 4000 {
            return Err(AppError::BadRequest);
        }

        Self::ensure_membership(state.conn(), user_id, chat_id).await?;

        let now = Utc::now();
        let message_record = MessageModel {
            id: Set(Uuid::new_v4().to_string()),
            chat_id: Set(chat_id.to_owned()),
            sender_id: Set(user_id.to_owned()),
            text: Set(trimmed.to_string()),
            created_at: Set(now.into()),
        };

        let message = message_record.insert(state.conn()).await?;

        let chat = Chat::find_by_id(chat_id.to_owned())
            .one(state.conn())
            .await?
            .ok_or(AppError::NotFound)?;

        let mut chat_active = chat.into_active_model();
        chat_active.last_message_id = Set(Some(message.id.clone()));
        chat_active.last_message_text = Set(Some(message.text.clone()));
        chat_active.last_message_time = Set(Some(message.created_at));
        let chat = chat_active.update(state.conn()).await?;

        let members = ChatMember::find()
            .filter(chat_members::Column::ChatId.eq(chat_id.to_owned()))
            .all(state.conn())
            .await?;

        for member in members {
            let mut active = member.clone().into_active_model();
            if member.user_id == user_id {
                active.unread_count = Set(0);
                active.last_read_message_id = Set(Some(message.id.clone()));
                active.last_read_at = Set(Some(now.into()));
            } else {
                active.unread_count = Set(member.unread_count + 1);
            }
            active.update(state.conn()).await?;
        }

        let chat_preview = ChatService::build_preview_for_user(state.conn(), chat, user_id).await?;

        Ok(SendMessageResponse {
            message: Self::map_message(&message),
            chat: chat_preview,
        })
    }

    async fn ensure_membership(
        conn: &DatabaseConnection,
        user_id: &str,
        chat_id: &str,
    ) -> Result<(), AppError> {
        let exists = ChatMember::find()
            .filter(chat_members::Column::ChatId.eq(chat_id.to_owned()))
            .filter(chat_members::Column::UserId.eq(user_id.to_owned()))
            .one(conn)
            .await?
            .is_some();

        if !exists {
            return Err(AppError::Forbidden);
        }

        Ok(())
    }

    fn map_message(message: &messages::Model) -> MessageDto {
        MessageDto {
            id: message.id.clone(),
            chat_id: message.chat_id.clone(),
            sender_id: message.sender_id.clone(),
            text: message.text.clone(),
            timestamp: message.created_at.into(),
        }
    }
}
