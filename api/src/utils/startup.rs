use crate::{config::GLOBAL_CONFIG, error::AppError, utils::security::hash_password};
use entity::{
    role::Role,
    users::{self, Entity as User},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DbConn};

pub async fn create_admin(db: DbConn) -> Result<(), AppError> {
    let user: Option<users::Model> = User::find_by_email(GLOBAL_CONFIG.admin_username.clone())
        .one(&db)
        .await?;

    if let None = user {
        let user = users::ActiveModel {
            first_name: Set("Живая".to_owned()),
            last_name: Set("Легенда".to_owned()),
            roles: Set(vec![Role::Admin, Role::Initiator]),
            email: Set(GLOBAL_CONFIG.admin_username.clone()),
            password: Set(hash_password(&GLOBAL_CONFIG.admin_password)?),
            ..Default::default()
        };

        user.insert(&db).await?;
    }

    Ok(())
}
