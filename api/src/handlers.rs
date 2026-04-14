use crate::{AppState, config::GLOBAL_CONFIG};
use axum::Router;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub mod auth;
pub mod chats;
pub mod profile;
pub mod users;

pub fn main_router() -> Router<AppState> {
    let avatar_dir = PathBuf::from(GLOBAL_CONFIG.avatar_path.clone());

    Router::new()
        .nest("/auth", auth::auth_router())
        .nest("/profile", profile::profile_router())
        .nest("/users", users::users_router())
        .nest("/chats", chats::chats_router())
        .nest_service("/images/avatar", ServeDir::new(avatar_dir))
}
