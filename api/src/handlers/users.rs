use crate::{AppState, dtos::users::UserDto, error::AppError, services::users::UserService};
use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};

pub fn users_router() -> Router<AppState> {
    Router::new().route("/by-nickname/{nickname}", get(find_by_nickname))
}

async fn find_by_nickname(
    State(state): State<AppState>,
    Path(nickname): Path<String>,
) -> Result<UserDto, AppError> {
    let user = UserService::find_by_nickname(&state, &nickname).await?;
    Ok(user)
}
