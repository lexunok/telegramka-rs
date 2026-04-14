use crate::{AppState, dtos::users::UserDto, error::AppError};
use entity::prelude::*;
use sea_orm::{EntityTrait, prelude::Uuid};

pub struct ProfileService;

impl ProfileService {
    pub async fn get_my(state: &AppState, user_id: Uuid) -> Result<UserDto, AppError> {
        let user = Users::find_by_id(user_id)
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        Ok(user)
    }
}
