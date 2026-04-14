use crate::{
    AppState,
    dtos::auth::{
        CheckEmailRequest, CheckEmailResponse, LoginRequest, RefreshRequest, RefreshResponse,
        RegisterRequest, RegisterResponse, VerificationInfo, VerifyCodeRequest, VerifyCodeResponse,
    },
    error::AppError,
    utils::{
        security::{TokenPair, build_token_pair},
        smtp::send_auth_code,
    },
};
use argon2::password_hash::rand_core::{OsRng, RngCore};
use chrono::{Duration, Utc};
use entity::{prelude::*, users, verification_codes};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, prelude::Expr,
};

pub struct AuthService;

impl AuthService {
    const CODE_TTL_MINUTES: i64 = 10;
    const CODE_LENGTH: usize = 6;

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

        let verification = VerificationCodes::find()
            .filter(verification_codes::Column::Email.eq(payload.email.to_lowercase().clone()))
            .filter(verification_codes::Column::Code.eq(payload.code))
            .filter(verification_codes::Column::ExpiresAt.gt(now))
            .order_by_desc(verification_codes::Column::CreatedAt)
            .one(&state.conn)
            .await?
            .ok_or(AppError::WrongCredentials)?;

        let mut verification_active = verification.clone().into_active_model();
        verification_active.attempt_count = Set(verification.attempt_count + 1);
        verification_active.used_at = Set(Some(now.into()));
        verification_active.update(state.conn()).await?;

        let user = User::find()
            .filter(users::Column::Email.eq(email.clone()))
            .one(state.conn())
            .await?
            .ok_or(AppError::NotFound)?;

        let tokens = Self::build_tokens(&user)?;

        Ok(VerifyCodeResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
            user: Self::map_user(&user),
        })
    }

    pub async fn refresh(
        state: &AppState,
        payload: RefreshRequest,
    ) -> Result<RefreshResponse, AppError> {
        let claims = decode_refresh_token(&payload.refresh_token)?;
        let user = User::find()
            .filter(users::Column::Id.eq(claims.sub))
            .one(state.conn())
            .await?
            .ok_or(AppError::NotFound)?;

        let tokens = Self::build_tokens(&user)?;

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
            code: Set(code.clone()),
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
