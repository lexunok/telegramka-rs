use crate::{config::GLOBAL_CONFIG, error::AppError};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header, Validation, decode, encode};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub name: String,
    pub nickname: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: TokenType,
}

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer ").map(str::to_string))
            .ok_or(AppError::WrongCredentials)?;

        let token_data = decode::<Claims>(
            &token,
            &GLOBAL_CONFIG.decoding_key,
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AppError::InvalidToken)?;

        if token_data.claims.token_type != TokenType::Access {
            return Err(AppError::InvalidToken);
        }

        Ok(token_data.claims)
    }
}

pub fn build_token_pair(
    id: Uuid,
    name: String,
    email: String,
    nickname: String,
) -> Result<TokenPair, AppError> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;

    let access_exp = (now + Duration::minutes(15)).timestamp() as usize;
    let refresh_exp = (now + Duration::days(7)).timestamp() as usize;

    let access_claims = Claims {
        sub: id,
        email,
        name,
        nickname,
        exp: access_exp,
        iat,
        token_type: TokenType::Access,
    };

    let refresh_claims = Claims {
        exp: refresh_exp,
        token_type: TokenType::Refresh,
        ..access_claims.clone()
    };

    let access_token = encode(
        &Header::new(Algorithm::HS256),
        &access_claims,
        &GLOBAL_CONFIG.encoding_key,
    )
    .map_err(|_| AppError::TokenCreation)?;

    let refresh_token = encode(
        &Header::new(Algorithm::HS256),
        &refresh_claims,
        &GLOBAL_CONFIG.encoding_key,
    )
    .map_err(|_| AppError::TokenCreation)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        expires_in: access_exp,
    })
}
pub fn decode_refresh_token(token: &String) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        &token,
        &GLOBAL_CONFIG.decoding_key,
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| AppError::InvalidToken)?;

    if token_data.claims.token_type != TokenType::Refresh {
        return Err(AppError::InvalidToken);
    }

    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError)?;
    Ok(password_hash.to_string())
}
pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
