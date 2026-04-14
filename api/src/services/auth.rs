use crate::{
    AppState,
    config::GLOBAL_CONFIG,
    dtos::auth::{LoginPayload, RegisterPayload},
    error::AppError,
    utils::security::{Claims, TokenType, hash_password, verify_password},
};
use axum_extra::extract::CookieJar;
use chrono::Local;
use entity::{
    invitation::{self, Entity as Invitation},
    users::{self, Entity as User},
};
use jsonwebtoken::{Validation, decode};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    TransactionTrait, prelude::Uuid,
};
use serde_json::json;
use validator::Validate;

pub struct AuthService;

impl AuthService {
    pub async fn refresh(jar: CookieJar) -> Result<Claims, AppError> {
        let refresh_cookie = jar.get("refresh_token").ok_or(AppError::WrongCredentials)?;

        let refresh_token = refresh_cookie.value();

        let token_data = decode::<Claims>(
            refresh_token,
            &GLOBAL_CONFIG.decoding_key,
            &Validation::default(),
        )
        .map_err(|_| AppError::InvalidToken)?;

        if token_data.claims.token_type != TokenType::Refresh {
            return Err(AppError::InvalidToken);
        }

        Ok(token_data.claims)
    }

    pub async fn login(state: &AppState, payload: LoginPayload) -> Result<users::Model, AppError> {
        payload.validate()?;

        let user = User::find_by_email(payload.email.to_lowercase())
            .one(&state.conn)
            .await?
            .ok_or(AppError::WrongCredentials)?;

        if !verify_password(&user.password, &payload.password) {
            return Err(AppError::WrongCredentials);
        }

        Ok(user)
    }

    pub async fn register_user(
        state: &AppState,
        invitation_id: Uuid,
        payload: RegisterPayload,
    ) -> Result<users::Model, AppError> {
        payload.validate()?;

        let txn = state.conn.begin().await?;

        let invitation = Invitation::find_by_id(invitation_id)
            .filter(invitation::Column::ExpiryDate.gt(Local::now()))
            .one(&txn)
            .await?
            .ok_or(AppError::NotFound)?;

        let mut user =
            users::ActiveModel::from_json(json!(payload)).map_err(|_| AppError::BadRequest)?;

        user.email = Set(invitation.email.to_lowercase());
        user.password = Set(hash_password(&payload.password)?);
        user.roles = Set(invitation.roles.clone());

        let user = user.insert(&txn).await?;

        let mut invitation = invitation.into_active_model();

        invitation.expiry_date = Set(Local::now().into());

        invitation.update(&txn).await?;

        txn.commit().await?;

        Ok(user)
    }
}
