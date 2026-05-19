use sea_orm_migration::{prelude::*, sea_query::Expr};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PushDevices::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PushDevices::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(PushDevices::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(PushDevices::DeviceId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(PushDevices::Platform).string().not_null())
                    .col(ColumnDef::new(PushDevices::FcmToken).text().not_null())
                    .col(ColumnDef::new(PushDevices::AppVersion).string())
                    .col(
                        ColumnDef::new(PushDevices::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(PushDevices::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PushDevices::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PushDevices::LastSeenAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PushDevices::Table, PushDevices::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum PushDevices {
    #[iden = "push_devices"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "user_id"]
    UserId,
    #[iden = "device_id"]
    DeviceId,
    #[iden = "platform"]
    Platform,
    #[iden = "fcm_token"]
    FcmToken,
    #[iden = "app_version"]
    AppVersion,
    #[iden = "is_active"]
    IsActive,
    #[iden = "created_at"]
    CreatedAt,
    #[iden = "updated_at"]
    UpdatedAt,
    #[iden = "last_seen_at"]
    LastSeenAt,
}

#[derive(Iden)]
enum Users {
    #[iden = "users"]
    Table,
    #[iden = "id"]
    Id,
}
