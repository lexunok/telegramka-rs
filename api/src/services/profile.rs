use std::path::PathBuf;

use crate::{AppState, config::GLOBAL_CONFIG, dtos::users::UserDto, error::AppError};
use axum::body::Bytes;
use entity::{prelude::*, users};
use image::ImageFormat;
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter,
    prelude::{Expr, Uuid},
};

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
    pub async fn upload_avatar(
        state: &AppState,
        user_id: Uuid,
        bytes: Bytes,
    ) -> Result<PathBuf, AppError> {
        let avatar_dir = PathBuf::from(&GLOBAL_CONFIG.avatar_path);
        let file_path = avatar_dir.join(format!("{}.webp", user_id));

        let img = image::load_from_memory(&bytes)
            .map_err(|_| AppError::Custom("Ошибка при загрузке изображения".to_string()))?;

        img.save_with_format(file_path.clone(), ImageFormat::WebP)
            .map_err(|_| AppError::Custom("Ошибка при сохранении аватара".to_string()))?;

        Users::update_many()
            .col_expr(users::Column::AvatarUrl, Expr::value(file_path.to_str()))
            .filter(users::Column::Id.eq(user_id))
            .exec(&state.conn)
            .await?;

        Ok(file_path)
    }
}
