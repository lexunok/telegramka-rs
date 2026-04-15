use sea_orm_migration::{prelude::*, sea_query::Expr};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::Nickname)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Users::AvatarUrl).string())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Chats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Chats::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Chats::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ChatMembers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ChatMembers::ChatId).uuid().not_null())
                    .col(ColumnDef::new(ChatMembers::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(ChatMembers::LastReadAt).timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(ChatMembers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(ChatMembers::ChatId)
                            .col(ChatMembers::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChatMembers::Table, ChatMembers::ChatId)
                            .to(Chats::Table, Chats::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChatMembers::Table, ChatMembers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Messages::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Messages::ChatId).uuid().not_null())
                    .col(ColumnDef::new(Messages::SenderId).uuid().not_null())
                    .col(ColumnDef::new(Messages::Text).text().not_null())
                    .col(
                        ColumnDef::new(Messages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Messages::Table, Messages::ChatId)
                            .to(Chats::Table, Chats::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Messages::Table, Messages::SenderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(VerificationCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VerificationCodes::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(VerificationCodes::Email).string().not_null())
                    .col(ColumnDef::new(VerificationCodes::Code).string().not_null())
                    .col(
                        ColumnDef::new(VerificationCodes::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VerificationCodes::AttemptCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(VerificationCodes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    #[iden = "users"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "name"]
    Name,
    #[iden = "email"]
    Email,
    #[iden = "nickname"]
    Nickname,
    #[iden = "avatar_url"]
    AvatarUrl,
    #[iden = "created_at"]
    CreatedAt,
}

#[derive(Iden)]
enum Chats {
    #[iden = "chats"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "created_at"]
    CreatedAt,
}

#[derive(Iden)]
enum ChatMembers {
    #[iden = "chat_members"]
    Table,
    #[iden = "chat_id"]
    ChatId,
    #[iden = "user_id"]
    UserId,
    #[iden = "last_read_at"]
    LastReadAt,
    #[iden = "created_at"]
    CreatedAt,
}

#[derive(Iden)]
enum Messages {
    #[iden = "messages"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "chat_id"]
    ChatId,
    #[iden = "sender_id"]
    SenderId,
    #[iden = "text"]
    Text,
    #[iden = "created_at"]
    CreatedAt,
}

#[derive(Iden)]
enum VerificationCodes {
    #[iden = "verification_codes"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "email"]
    Email,
    #[iden = "code"]
    Code,
    #[iden = "expires_at"]
    ExpiresAt,
    #[iden = "attempt_count"]
    AttemptCount,
    #[iden = "created_at"]
    CreatedAt,
}
