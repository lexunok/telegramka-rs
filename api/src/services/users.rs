use crate::{AppState, dtos::users::UserDto, error::AppError};
use entity::prelude::*;
use sea_orm::{EntityTrait, QueryFilter};

pub struct UserService;

impl UserService {
    pub async fn find_by_nickname(state: &AppState, nickname: &str) -> Result<UserDto, AppError> {
        let user = Users::find()
            .filter(Users::COLUMN.nickname.eq(nickname))
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(user)
    }
}
