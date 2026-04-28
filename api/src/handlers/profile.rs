use crate::{
    AppState,
    dtos::users::{AvatarResponse, UserDto},
    error::AppError,
    services::profile::ProfileService,
    utils::security::Claims,
};
use axum::{
    Router,
    extract::{Multipart, State},
    routing::{get, post},
};
use futures::StreamExt;

pub fn profile_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_my_profile))
        .route("/avatar", post(upload_avatar))
}

async fn get_my_profile(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<UserDto, AppError> {
    let user = ProfileService::get_my(&state, claims.sub).await?;
    Ok(user)
}
async fn upload_avatar(
    State(state): State<AppState>,
    claims: Claims,
    mut multipart: Multipart,
) -> Result<AvatarResponse, AppError> {
    println!("Uploading avatar");
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest)?
    {
        let field_name = field.name().unwrap_or("").to_string();
        println!("Field Name: {}", field_name);

        if field_name == "avatar" {
            // let bytes = field.bytes().await.map_err(|_| AppError::BadRequest)?;
            let mut data = Vec::new();
            let mut stream = field;
            println!("Reading stream");
            while let Some(chunk) = stream.next().await {
                println!("Reading chunk");
                let chunk = chunk.map_err(|e| {
                    println!("Chunk error: {}", e);
                    AppError::BadRequest
                })?;
                data.extend_from_slice(&chunk);
            }
            let path = ProfileService::upload_avatar(&state, claims.sub, data).await?;
            return Ok(AvatarResponse { path });
        }
    }

    Err(AppError::BadRequest)
}
