use crate::{AppState, config::GLOBAL_CONFIG};
use axum::Router;
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub mod auth;

pub fn main_router() -> Router<AppState> {
    let avatar_dir = PathBuf::from(GLOBAL_CONFIG.avatar_path.clone());

    Router::new()
        .nest("/auth", auth::auth_router())
        .nest_service("/images/avatar", ServeDir::new(avatar_dir))
}
