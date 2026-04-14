use crate::{config::GLOBAL_CONFIG, error::AppError};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use chrono::{Duration, Utc};
use entity::role::Role;
use jsonwebtoken::{Header, Validation, decode, encode};
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
    pub first_name: String,
    pub last_name: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: TokenType,
    pub roles: Vec<Role>,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let access_token = jar
            .get("access_token")
            .ok_or(AppError::WrongCredentials)?
            .value()
            .to_string();

        let token_data = decode::<Claims>(
            &access_token,
            &GLOBAL_CONFIG.decoding_key,
            &Validation::default(),
        )
        .map_err(|_| AppError::InvalidToken)?;

        if token_data.claims.token_type != TokenType::Access {
            return Err(AppError::InvalidToken);
        }

        Ok(token_data.claims)
    }
}

pub fn generate_tokens(
    sub: Uuid,
    email: String,
    first_name: String,
    last_name: String,
    roles: Vec<Role>,
) -> Result<CookieJar, AppError> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(15)).timestamp() as usize;

    let claims = Claims {
        sub: sub.clone(),
        email: email.clone(),
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        iat,
        exp,
        token_type: TokenType::Access,
        roles: roles.clone(),
    };

    let access_token = encode(&Header::default(), &claims, &GLOBAL_CONFIG.encoding_key)
        .map_err(|_| AppError::TokenCreation)?;

    let exp = (now + Duration::days(7)).timestamp() as usize;

    let claims = Claims {
        sub,
        email,
        first_name,
        last_name,
        iat,
        exp,
        token_type: TokenType::Refresh,
        roles,
    };

    let refresh_token = encode(&Header::default(), &claims, &GLOBAL_CONFIG.encoding_key)
        .map_err(|_| AppError::TokenCreation)?;

    let is_secure: bool = !cfg!(debug_assertions);

    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_secure)
        .max_age(time::Duration::minutes(30));

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(is_secure)
        .max_age(time::Duration::days(30));

    Ok(CookieJar::new().add(access_cookie).add(refresh_cookie))
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
