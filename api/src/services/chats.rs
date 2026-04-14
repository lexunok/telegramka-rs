use crate::{
    AppState,
    dtos::chats::{ChatCreateResponse, ChatListResponse, ChatPreview, ChatReadResponse},
    error::AppError,
};
use chrono::Utc;
use entity::{
    chat_members::{self, ActiveModel as ChatMemberModel, Entity as ChatMember},
    chats::{self, ActiveModel as ChatModel, Entity as Chat},
    users::{self, Entity as User},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder,
};
use uuid::Uuid;

pub struct ChatService;

impl ChatService {
    pub async fn list_chats(
        state: &AppState,
        user_id: &str,
        limit: Option<u32>,
    ) -> Result<ChatListResponse, AppError> {
        let members = ChatMember::find()
            .filter(chat_members::Column::UserId.eq(user_id.to_owned()))
            .order_by_desc(chat_members::Column::CreatedAt)
            .all(state.conn())
            .await?;

        let mut items = Vec::with_capacity(members.len());
        for member in members {
            if let Some(chat) = Chat::find_by_id(member.chat_id.clone())
                .one(state.conn())
                .await?
            {
                items.push(Self::build_preview_for_user(state.conn(), chat, user_id).await?);
            }
        }

        if let Some(limit) = limit {
            items.truncate(limit as usize);
        }

        Ok(ChatListResponse {
            items,
            next_cursor: None,
        })
    }

    pub async fn open_or_get_chat(
        state: &AppState,
        user_id: &str,
        nickname: &str,
    ) -> Result<ChatCreateResponse, AppError> {
        let normalized_nickname = Self::normalize_nickname(nickname)?;
        let target_user = User::find()
            .filter(users::Column::Nickname.eq(normalized_nickname.clone()))
            .one(state.conn())
            .await?
            .ok_or(AppError::NotFound)?;

        if target_user.id == user_id {
            return Err(AppError::BadRequest);
        }

        let existing = ChatMember::find()
            .filter(chat_members::Column::UserId.eq(user_id.to_owned()))
            .all(state.conn())
            .await?;

        for member in existing {
            let partner = ChatMember::find()
                .filter(chat_members::Column::ChatId.eq(member.chat_id.clone()))
                .filter(chat_members::Column::UserId.eq(target_user.id.clone()))
                .one(state.conn())
                .await?;

            if partner.is_some() {
                let chat = Chat::find_by_id(member.chat_id)
                    .one(state.conn())
                    .await?
                    .ok_or(AppError::NotFound)?;

                let preview = Self::build_preview_for_user(state.conn(), chat, user_id).await?;

                return Ok(ChatCreateResponse {
                    chat: preview,
                    created: false,
                });
            }
        }

        Self::create_chat(state, user_id, &target_user).await
    }

    pub async fn get_chat(
        state: &AppState,
        user_id: &str,
        chat_id: &str,
    ) -> Result<ChatPreview, AppError> {
        let chat = Chat::find_by_id(chat_id.to_owned())
            .one(state.conn())
            .await?
            .ok_or(AppError::NotFound)?;

        Self::build_preview_for_user(state.conn(), chat, user_id).await
    }

    pub async fn mark_as_read(
        state: &AppState,
        user_id: &str,
        chat_id: &str,
        read_through_message_id: Option<String>,
    ) -> Result<ChatReadResponse, AppError> {
        let member = ChatMember::find()
            .filter(chat_members::Column::ChatId.eq(chat_id.to_owned()))
            .filter(chat_members::Column::UserId.eq(user_id.to_owned()))
            .one(state.conn())
            .await?
            .ok_or(AppError::Forbidden)?;

        let mut active = member.clone().into_active_model();
        active.unread_count = Set(0);
        active.last_read_message_id = Set(read_through_message_id);
        active.last_read_at = Set(Some(Utc::now().into()));

        active.update(state.conn()).await?;

        Ok(ChatReadResponse {
            ok: true,
            unread: 0,
        })
    }

    async fn create_chat(
        state: &AppState,
        user_id: &str,
        partner: &users::Model,
    ) -> Result<ChatCreateResponse, AppError> {
        let chat_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let chat = ChatModel {
            id: Set(chat_id.clone()),
            created_by: Set(user_id.to_owned()),
            kind: Set("direct".to_string()),
            is_private: Set(true),
            created_at: Set(now.into()),
            last_message_id: Set(None),
            last_message_text: Set(None),
            last_message_time: Set(None),
            ..Default::default()
        };

        let chat = chat.insert(state.conn()).await?;

        let first_member = ChatMemberModel {
            chat_id: Set(chat_id.clone()),
            user_id: Set(user_id.to_owned()),
            unread_count: Set(0),
            last_read_message_id: Set(None),
            last_read_at: Set(None),
            created_at: Set(now.into()),
        };

        let second_member = ChatMemberModel {
            chat_id: Set(chat_id.clone()),
            user_id: Set(partner.id.clone()),
            unread_count: Set(0),
            last_read_message_id: Set(None),
            last_read_at: Set(None),
            created_at: Set(now.into()),
        };

        first_member.insert(state.conn()).await?;
        second_member.insert(state.conn()).await?;

        let preview = Self::build_preview_for_user(state.conn(), chat, user_id).await?;

        Ok(ChatCreateResponse {
            chat: preview,
            created: true,
        })
    }

    pub(crate) async fn build_preview_for_user(
        conn: &DatabaseConnection,
        chat: chats::Model,
        user_id: &str,
    ) -> Result<ChatPreview, AppError> {
        let member = ChatMember::find()
            .filter(chat_members::Column::ChatId.eq(chat.id.clone()))
            .filter(chat_members::Column::UserId.eq(user_id.to_owned()))
            .one(conn)
            .await?
            .ok_or(AppError::Forbidden)?;

        let partner_member = ChatMember::find()
            .filter(chat_members::Column::ChatId.eq(chat.id.clone()))
            .filter(chat_members::Column::UserId.ne(user_id.to_owned()))
            .one(conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let partner = User::find_by_id(partner_member.user_id.clone())
            .one(conn)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(ChatPreview {
            id: chat.id.clone(),
            name: partner.name,
            nickname: partner.nickname,
            last_message: chat.last_message_text,
            last_message_time: chat.last_message_time.map(Into::into),
            unread: member.unread_count,
            avatar_url: partner.avatar_url,
        })
    }

    fn normalize_nickname(nickname: &str) -> Result<String, AppError> {
        let cleaned = nickname.trim();
        if cleaned.is_empty() {
            return Err(AppError::BadRequest);
        }

        let normalized = if cleaned.starts_with('@') {
            cleaned.to_lowercase()
        } else {
            format!("@{}", cleaned.to_lowercase())
        };

        if normalized.len() > 32 || normalized.len() < 2 {
            return Err(AppError::BadRequest);
        }

        if !normalized
            .chars()
            .all(|c| c == '@' || c == '_' || c.is_ascii_alphanumeric())
        {
            return Err(AppError::BadRequest);
        }

        Ok(normalized)
    }
}
