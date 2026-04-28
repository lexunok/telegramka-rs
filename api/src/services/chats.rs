use crate::{
    AppState,
    dtos::{
        chats::ChatResponse,
        messages::{MessageDto, MessageQuery, SendMessageRequest, WsEvent},
    },
    error::AppError,
};
use chrono::Utc;
use entity::{chat_members, chats, messages, prelude::*, users};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, EntityTrait, ExprTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, RelationTrait, TransactionTrait,
    prelude::{Uuid, *},
    sea_query::{Alias, IntoCondition, Query},
};
use std::cmp::min;

pub struct ChatService;

impl ChatService {
    pub async fn list_chats(state: &AppState, user_id: Uuid) -> Vec<ChatResponse> {
        let me = Alias::new("me");
        let other_member = Alias::new("other_member");

        Chats::find()
            .join_as(
                JoinType::InnerJoin,
                chats::Relation::ChatMembers
                    .def()
                    .on_condition(move |_left, right| {
                        Expr::col((right, chat_members::Column::UserId))
                            .eq(user_id)
                            .into_condition()
                    }),
                me.clone(),
            )
            .join_as(
                JoinType::InnerJoin,
                chats::Relation::ChatMembers
                    .def()
                    .on_condition(move |_left, right| {
                        Expr::col((right, chat_members::Column::ChatId))
                            .eq(Expr::col((me.clone(), chat_members::Column::ChatId)))
                            .into_condition()
                    }),
                other_member.clone(),
            )
            .join(
                JoinType::InnerJoin,
                chat_members::Relation::Users
                    .def()
                    .from_alias(other_member.clone())
                    .on_condition(move |_left, right| {
                        Expr::col((right, users::Column::Id))
                            .ne(user_id)
                            .into_condition()
                    }),
            )
            .select_only()
            .column(chats::Column::Id)
            .column_as(users::Column::Name, "name")
            .column_as(users::Column::Nickname, "nickname")
            .column_as(users::Column::AvatarUrl, "avatar_url")
            .expr_as(
                Messages::find()
                    .select_only()
                    .column(messages::Column::Text)
                    .filter(Expr::col(messages::Column::ChatId).equals(chats::Column::Id))
                    .order_by_desc(messages::Column::CreatedAt)
                    .limit(1)
                    .into_query(),
                "last_message",
            )
            .expr_as(
                Messages::find()
                    .select_only()
                    .column(messages::Column::CreatedAt)
                    .filter(Expr::col(messages::Column::ChatId).equals(chats::Column::Id))
                    .order_by_desc(messages::Column::CreatedAt)
                    .limit(1)
                    .into_query(),
                "last_message_time",
            )
            .expr_as(
                Query::select()
                    .expr(Expr::col(messages::Column::Id).count())
                    .from(Messages)
                    .and_where(Expr::col(messages::Column::ChatId).equals(chats::Column::Id))
                    .and_where(Expr::cust(format!(
                        "NOT EXISTS (
                                SELECT 1 FROM chat_members cm
                                WHERE cm.chat_id = chats.id
                                AND cm.user_id = '{}'
                                AND cm.last_read_at >= messages.created_at
                            )",
                        user_id
                    )))
                    .to_owned(),
                "unread",
            )
            .order_by_desc(Expr::col("last_message_time"))
            .into_model::<ChatResponse>()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }
    pub async fn list_messages(
        state: &AppState,
        user_id: Uuid,
        chat_id: Uuid,
        params: MessageQuery,
    ) -> Vec<MessageDto> {
        let limit = min(params.limit.unwrap_or(50), 100);
        // Нужна еще проверка что я есть в чате
        let mut query = Messages::find().filter(messages::Column::ChatId.eq(chat_id));

        if let Some(before) = params.before {
            query = query.filter(messages::Column::CreatedAt.lt(before));
        }

        let mut messages = query
            .order_by_desc(messages::Column::CreatedAt)
            .limit(limit)
            .into_model::<MessageDto>()
            .all(&state.conn)
            .await
            .unwrap_or_default();

        messages.reverse();

        if params.before.is_none() && !messages.is_empty() {
            let _ = ChatMembers::update_many()
                .col_expr(chat_members::Column::LastReadAt, Expr::current_timestamp())
                .filter(chat_members::Column::ChatId.eq(chat_id))
                .filter(chat_members::Column::UserId.eq(user_id))
                .exec(&state.conn)
                .await;
        }

        messages
    }
    pub async fn send_message(
        state: &AppState,
        sender_id: Uuid,
        payload: SendMessageRequest,
    ) -> Result<MessageDto, AppError> {
        // есть ли чат (только для персональных)
        let chat_id = ChatMembers::find()
            .select_only()
            .column(chat_members::Column::ChatId)
            .filter(chat_members::Column::ChatId.eq(payload.chat_id))
            // .filter(chat_members::Column::UserId.is_in(vec![sender_id, payload.user_id]))
            .group_by(chat_members::Column::ChatId)
            .into_tuple::<Uuid>()
            .one(&state.conn)
            .await?;

        let txn = state.conn.begin().await?;

        let chat_id = if let Some(chat_id) = chat_id {
            chat_id
        } else {
            let chat = Chats::insert(chats::ActiveModel {
                ..Default::default()
            })
            .exec(&txn)
            .await?;

            let chat_id = chat.last_insert_id;

            ChatMembers::insert(chat_members::ActiveModel {
                chat_id: Set(chat_id),
                user_id: Set(sender_id),
                ..Default::default()
            })
            .exec(&txn)
            .await?;

            ChatMembers::insert(chat_members::ActiveModel {
                chat_id: Set(chat_id),
                user_id: Set(payload.chat_id),
                ..Default::default()
            })
            .exec(&txn)
            .await?;

            chat_id
        };

        let message = Messages::insert(messages::ActiveModel {
            id: Set(payload.id),
            chat_id: Set(chat_id),
            sender_id: Set(sender_id),
            text: Set(payload.text.clone()),
            ..Default::default()
        })
        .exec(&txn)
        .await?;

        let message_id = message.last_insert_id;
        let now = Utc::now();

        let dto = MessageDto {
            id: message_id,
            chat_id,
            sender_id,
            text: payload.text,
            created_at: now,
        };

        ChatMembers::update_many()
            .col_expr(chat_members::Column::LastReadAt, Expr::value(now))
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .filter(chat_members::Column::UserId.eq(sender_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        //Пока только персональные чаты
        let recipients: Vec<Uuid> = vec![chat_id, sender_id];

        let _ = state.tx.send(WsEvent::NewMessage {
            recipients,
            message: dto.clone(),
        });

        Ok(dto)
    }
}
