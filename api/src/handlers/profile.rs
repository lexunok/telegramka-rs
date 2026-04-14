use crate::{
    AppState, dtos::users::UserDto, error::AppError, services::profile::ProfileService,
    utils::security::Claims,
};
use axum::{Router, extract::State, routing::get};

pub fn profile_router() -> Router<AppState> {
    Router::new().route("/", get(get_my_profile))
}

async fn get_my_profile(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<UserDto, AppError> {
    let user = ProfileService::get_my(&state, claims.sub).await?;
    Ok(user)
}
