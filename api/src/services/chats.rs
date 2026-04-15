use crate::{
    AppState, dtos::{chats::ChatResponse, messages::{MessageDto, MessageQuery}}, error::AppError,
};
use chrono::Utc;
use entity::{chat_members, chats, messages, prelude::*, users};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, ExprTrait, JoinType, QueryFilter, QueryOrder, QuerySelect, QueryTrait, RelationTrait, prelude::{Uuid, *}, sea_query::Query
};

pub struct ChatService;

impl ChatService {
    pub async fn list_chats(
        state: &AppState,
        user_id: Uuid,
    ) -> Vec<ChatResponse> {
        let last_message_subquery = Messages::find()
            .select_only()
            .column(messages::Column::Text)
            .filter(Expr::col((Messages, messages::Column::ChatId)).equals((Chats, chats::Column::Id)))
            .order_by_desc(messages::Column::CreatedAt)
            .limit(1)
            .into_query();

        let last_message_time_subquery = Messages::find()
            .select_only()
            .column(messages::Column::CreatedAt)
            .filter(Expr::col((Messages, messages::Column::ChatId)).equals((Chats, chats::Column::Id)))
            .order_by_desc(messages::Column::CreatedAt)
            .limit(1)
            .into_query();

        let unread_subquery = Query::select()
            .expr(Expr::col(messages::Column::Id).count())
            .from(Messages)
            .and_where(
                Expr::col((Messages, messages::Column::ChatId))
                    .equals((Chats, chats::Column::Id))
            )
            .and_where(
                Expr::cust(
                    "chat_members.last_read_at IS NULL OR messages.created_at > chat_members.last_read_at"
                )
            )
            .to_owned();

        ChatMembers::find()
            .filter(chat_members::Column::UserId.eq(user_id))

            .inner_join(Chats)

            .join(
                JoinType::InnerJoin,
                chat_members::Relation::Users.def().rev(), 
            )
            .filter(users::Column::Id.ne(user_id)) 

            .select_only()
            .column_as(chats::Column::Id, "id")
            .column_as(users::Column::Name, "name")
            .column_as(users::Column::Nickname, "nickname")
            .column_as(users::Column::AvatarUrl, "avatar_url")
    
            .expr_as(unread_subquery, "unread")

            .expr_as(last_message_subquery, "last_message")
            .expr_as(last_message_time_subquery, "last_message_time")

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

        let mut query = Messages::find()
            .filter(messages::Column::ChatId.eq(chat_id));

        if let Some(cursor) = params.before {
            query = query.filter(messages::Column::CreatedAt.lt(cursor));
        }

        let messages = query
            .order_by_desc(messages::Column::CreatedAt)
            .limit(params.limit)
            .into_model::<MessageDto>()
            .all(&state.conn)
            .await
            .unwrap_or_default();

        if params.before.is_none() && !messages.is_empty() {
            ChatMembers::update_many()
                .col_expr(
                    chat_members::Column::LastReadAt,
                    Expr::current_timestamp(),
                )
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
        recipient_id: Uuid,
        text: String,
    ) -> Result<MessageDto, AppError> {

        // есть ли чат (только для персональных)
        let chat_id = ChatMembers::find()
            .select_only()
            .column(chat_members::Column::ChatId)
            .filter(
                chat_members::Column::UserId
                    .is_in(vec![sender_id, recipient_id])
            )
            .group_by(chat_members::Column::ChatId)
            .having(
                Expr::col(chat_members::Column::UserId)
                    .count()
                    .eq(2)
            )
            .into_tuple::<Uuid>()
            .one(&state.conn)
            .await?;

        let chat_id = if let Some(chat_id) = chat_id {
            chat_id
        } else {
            let chat = Chats::insert(chats::ActiveModel {..Default::default()})
                .exec(&state.conn)
                .await?;

            let chat_id = chat.last_insert_id;

            ChatMembers::insert(chat_members::ActiveModel {
                chat_id: Set(chat_id),
                user_id: Set(sender_id),
                ..Default::default()
            })
            .exec(&state.conn)
            .await?;

            ChatMembers::insert(chat_members::ActiveModel {
                chat_id: Set(chat_id),
                user_id: Set(recipient_id),
                ..Default::default()
            })
            .exec(&state.conn)
            .await?;

            chat_id
        };


        let message = Messages::insert(messages::ActiveModel {
            chat_id: Set(chat_id),
            sender_id: Set(sender_id),
            text: Set(text.clone()),
            ..Default::default()
        })
            .exec(&state.conn)
            .await?;

        let message_id = message.last_insert_id;
        let now = Utc::now();

        ChatMembers::update_many()
            .col_expr(
                chat_members::Column::LastReadAt,
                Expr::value(now),
            )
            .filter(chat_members::Column::ChatId.eq(chat_id))
            .filter(chat_members::Column::UserId.eq(sender_id))
            .exec(&state.conn)
            .await?;

        Ok(MessageDto {
            id: message_id,
            chat_id,
            sender_id,
            text,
            created_at: now,
        })
    }
}
