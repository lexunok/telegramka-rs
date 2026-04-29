use crate::{
    AppState,
    dtos::{
        chats::ChatResponse,
        messages::{MessageDto, MessageQuery, SendMessageRequest, WsEnvelope, WsEvent},
    },
    error::AppError,
};
use chrono::Utc;
use entity::{chat_members, chats, messages, prelude::*, users};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, EntityTrait, ExprTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait, TransactionTrait, prelude::{Uuid, *}, sea_query::{Alias,Func, IntoCondition, Query}
};
use std::cmp::min;

pub struct ChatService;

impl ChatService {
    pub async fn list_chats(state: &AppState, user_id: Uuid) -> Vec<ChatResponse> {
        let me = Alias::new("me");
        let other_member = Alias::new("other_member");
        let m = Alias::new("m");

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
            .join_as(
                JoinType::LeftJoin,
                chats::Relation::Messages.def(),
                m.clone(),
            )
            .select_only()
            .column(chats::Column::Id)
            .column_as(users::Column::Id, "user_id")
            .column_as(users::Column::Name, "name")
            .column_as(users::Column::Nickname, "nickname")
            .column_as(users::Column::AvatarUrl, "avatar_url")
            .expr_as(
                Query::select()
                    .column(messages::Column::Text)
                    .from(Messages)
                    .and_where(
                        Expr::col(messages::Column::ChatId)
                            .eq(Expr::cust("chats.id"))       
                    )
                    .order_by(messages::Column::CreatedAt, sea_orm::Order::Desc)
                    .limit(1)
                    .to_owned(),
                "last_message",
            )
            .expr_as(
                Query::select()
                    .column(messages::Column::CreatedAt)
                    .from(Messages)
                    .and_where(
                        Expr::col(messages::Column::ChatId)
                            .eq(Expr::cust("chats.id"))       
                     )
                    .order_by(messages::Column::CreatedAt, sea_orm::Order::Desc)
                    .limit(1)
                    .to_owned(),
                "last_message_time",
            )
            .expr_as(
                Expr::expr(
                    Func::sum(
                        Expr::case(
                            Condition::any()
                                .add(
                                    Expr::col((Alias::new("me"), chat_members::Column::LastReadAt)).is_null()
                                )
                                .add(
                                    Expr::col((m.clone(), messages::Column::CreatedAt))
                                        .gt(Expr::col((Alias::new("me"), chat_members::Column::LastReadAt)))
                                ),
                            1,
                        )
                        .finally(0)
                    )
                ),
                "unread",
            )
            .group_by(chats::Column::Id)
            .group_by(users::Column::Id)
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

        let chat_id = ChatMembers::find()
            .select_only()
            .column(chat_members::Column::ChatId)
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .filter(chat_members::Column::UserId.eq(user_id))
            .group_by(chat_members::Column::ChatId)
            .into_tuple::<Uuid>()
            .one(&state.conn)
            .await;

        let chat_id = match chat_id {
            Ok(Some(id)) => id,
            Ok(None) | Err(_) => return vec![],
        };

        let limit = min(params.limit.unwrap_or(50), 100);

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
            let read_at = Utc::now();

            let _ = ChatMembers::update_many()
                .col_expr(chat_members::Column::LastReadAt, Expr::value(read_at))
                .filter(chat_members::Column::ChatId.eq(chat_id))
                .filter(chat_members::Column::UserId.eq(user_id))
                .exec(&state.conn)
                .await;

            if let Ok(recipients) = ChatMembers::find()
                .filter(chat_members::Column::ChatId.eq(chat_id))
                .select_only()
                .column(chat_members::Column::UserId)
                .into_tuple::<Uuid>()
                .all(&state.conn)
                .await
            {
                let _ = state.tx.send(WsEnvelope {
                    recipients: Some(recipients),
                    event: WsEvent::Read {
                        chat_id,
                        user_id,
                        read_at,
                    },
                });
            }
        }

        messages
    }
    pub async fn mark_read(state: &AppState, user_id: Uuid, chat_id: Uuid) -> bool {
        let is_member = ChatMembers::find()
            .select_only()
            .column(chat_members::Column::ChatId)
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .filter(chat_members::Column::UserId.eq(user_id))
            .into_tuple::<Uuid>()
            .one(&state.conn)
            .await
            .ok()
            .flatten()
            .is_some();

        if !is_member {
            return false;
        }

        let read_at = Utc::now();

        if ChatMembers::update_many()
            .col_expr(chat_members::Column::LastReadAt, Expr::value(read_at))
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .filter(chat_members::Column::UserId.eq(user_id))
            .exec(&state.conn)
            .await
            .is_err()
        {
            return false;
        }

        if let Ok(recipients) = ChatMembers::find()
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .select_only()
            .column(chat_members::Column::UserId)
            .into_tuple::<Uuid>()
            .all(&state.conn)
            .await
        {
            let _ = state.tx.send(WsEnvelope {
                recipients: Some(recipients),
                event: WsEvent::Read {
                    chat_id,
                    user_id,
                    read_at,
                },
            });
        }

        true
    }
    pub async fn send_message(
        state: &AppState,
        sender_id: Uuid,
        payload: SendMessageRequest,
    ) -> Result<MessageDto, AppError> {

        let txn = state.conn.begin().await?;

        let chat_id = if let Some(chat_id) = payload.chat_id {
            ChatMembers::find()
                .select_only()
                .column(chat_members::Column::ChatId)
                .filter(chat_members::Column::ChatId.eq(chat_id))
                .filter(chat_members::Column::UserId.eq(sender_id))
                .group_by(chat_members::Column::ChatId)
                .into_tuple::<Uuid>()
                .one(&txn)
                .await?
                .ok_or(AppError::NotFound)?

        } else if let Some(user_id) = payload.user_id {

            // Чисто в теории можно создать дубликат чата, поэтому стоит сделать какую то проверку чтоль
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
                user_id: Set(user_id),
                ..Default::default()
            })
            .exec(&txn)
            .await?;

            chat_id
        } else {
            return Err(AppError::BadRequest);
        };

        let message: messages::Model = messages::ActiveModel {
            id: Set(payload.id),
            chat_id: Set(chat_id),
            sender_id: Set(sender_id),
            text: Set(payload.text.clone()),
            ..Default::default()
        }.insert(&txn).await?;

        let dto = MessageDto {
            id: message.id,
            chat_id,
            sender_id,
            text: message.text,
            created_at: message.created_at.into(),
        };

        ChatMembers::update_many()
            .col_expr(chat_members::Column::LastReadAt, Expr::value(dto.created_at))
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .filter(chat_members::Column::UserId.eq(sender_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        let recipients = ChatMembers::find()
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .select_only()
            .column(chat_members::Column::UserId)
            .into_tuple::<Uuid>()
            .all(&state.conn)
            .await?;

        let _ = state.tx.send(WsEnvelope {
            recipients: Some(recipients),
            event: WsEvent::NewMessage {
                message: dto.clone(),
            },
        });

        Ok(dto)
    }
}
