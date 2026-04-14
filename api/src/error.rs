use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Wrong credentials")]
    WrongCredentials,

    #[error("Token creation error")]
    TokenCreation,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Not Found")]
    NotFound,

    #[error("Bad Request")]
    BadRequest,

    #[error("Forbidden")]
    Forbidden,

    #[error("{0}")]
    Custom(String),

    #[error("Internal Server Error")]
    InternalServerError,

    #[error("Validation error")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("Database error")]
    DbErr(
        #[from]
        #[source]
        sea_orm::DbErr,
    ),

    #[error("An error occurred with Redis")]
    RedisErr(
        #[from]
        #[source]
        redis::RedisError,
    ),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::WrongCredentials | AppError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AppError::TokenCreation | AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::Custom(msg) => (StatusCode::BAD_REQUEST, msg.clone()),

            AppError::ValidationError(e) => {
                let errors = e
                    .field_errors()
                    .into_iter()
                    .map(|(field, errors)| {
                        let messages = errors
                            .iter()
                            .map(|err| err.message.as_ref().unwrap().to_string())
                            .collect::<Vec<_>>();
                        (field, messages)
                    })
                    .collect::<serde_json::Value>();
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({ "errors": errors })),
                )
                    .into_response();
            }

            AppError::DbErr(e) => {
                tracing::error!("Database source error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred".to_string(),
                )
            }
            AppError::RedisErr(e) => {
                tracing::error!("Redis source error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal service error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
