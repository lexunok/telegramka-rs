use crate::{
    AppState,
    dtos::{
        auth::{
            RefreshRequest, RefreshResponse, RegisterRequest, VerifyCodeRequest, VerifyCodeResponse,
        },
        users::UserDto,
    },
    error::AppError,
    utils::{
        security::{build_token_pair, decode_refresh_token, hash_password, verify_password},
        smtp::send_auth_code,
    },
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use chrono::{Duration, Utc};
use entity::{prelude::*, users, verification_codes};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder,
};

pub struct AuthService;

impl AuthService {
    const CODE_TTL_MINUTES: i64 = 10;

    pub async fn login(state: &AppState, email: String) -> Result<(), AppError> {
        let email = email.to_lowercase();
        let _ = Users::find_by_email(&email)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        Self::issue_verification_code(state, email).await?;

        Ok(())
    }

    pub async fn register(state: &AppState, payload: RegisterRequest) -> Result<(), AppError> {
        let user = users::ActiveModel {
            email: Set(payload.email.to_lowercase()),
            name: Set(payload.name),
            nickname: Set(payload.nickname),
            ..Default::default()
        };

        let user = user.insert(&state.conn).await?;

        Self::issue_verification_code(state, user.email).await?;
        Ok(())
    }

    pub async fn verify_code(
        state: &AppState,
        payload: VerifyCodeRequest,
    ) -> Result<VerifyCodeResponse, AppError> {
        let now = Utc::now();
        let email = payload.email.to_lowercase();
        let verification = VerificationCodes::find()
            .filter(verification_codes::Column::Email.eq(email.clone()))
            .filter(verification_codes::Column::ExpiresAt.gt(now))
            .filter(verification_codes::Column::AttemptCount.lte(3))
            .order_by_desc(verification_codes::Column::CreatedAt)
            .one(&state.conn)
            .await?
            .ok_or(AppError::WrongCredentials)?;

        if !verify_password(&verification.code, &payload.code) {
            let attempt_count = verification.attempt_count;
            let mut verification = verification.into_active_model();
            verification.attempt_count = Set(attempt_count + 1);
            verification.update(&state.conn).await?;
            return Err(AppError::Custom("Неправильно введен код!".to_string()));
        }

        let user: UserDto = Users::find()
            .filter(users::Column::Email.eq(email))
            .into_partial_model()
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let tokens = build_token_pair(
            user.id,
            user.name.clone(),
            user.email.clone(),
            user.nickname.clone(),
        )?;

        Ok(VerifyCodeResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
            user,
        })
    }

    pub async fn refresh(
        state: &AppState,
        payload: RefreshRequest,
    ) -> Result<RefreshResponse, AppError> {
        let claims = decode_refresh_token(&payload.refresh_token)?;
        let user = Users::find_by_id(claims.sub)
            .one(&state.conn)
            .await?
            .ok_or(AppError::NotFound)?;

        let tokens = build_token_pair(user.id, user.name, user.email, user.nickname)?;

        Ok(RefreshResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
        })
    }

    async fn issue_verification_code(state: &AppState, email: String) -> Result<(), AppError> {
        let mut rng = OsRng;
        let random_u32 = rng.next_u32();
        let code = (100_000 + (random_u32 % 900_000)).to_string();

        let now = Utc::now();
        let expires_at = now + Duration::minutes(Self::CODE_TTL_MINUTES);

        let verification = verification_codes::ActiveModel {
            email: Set(email.clone()),
            code: Set(hash_password(&code)?),
            expires_at: Set(expires_at.into()),
            ..Default::default()
        };

        verification.insert(&state.conn).await?;

        send_auth_code(code, email)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }
}
