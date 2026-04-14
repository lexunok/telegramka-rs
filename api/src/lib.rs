use crate::{config::GLOBAL_CONFIG, handlers::main_router};
use axum::Router;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::fs;

mod config;
mod dtos;
mod error;
mod handlers;
mod services;
mod utils;

#[tokio::main]
pub async fn start() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    dotenvy::dotenv().ok();

    let state = init_app_state().await?;

    let app = build_app(state)?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", GLOBAL_CONFIG.port)).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
}

pub async fn init_app_state() -> anyhow::Result<AppState> {
    let conn = Database::connect(GLOBAL_CONFIG.db_url.to_owned()).await?;
    Migrator::up(&conn, None).await?;

    Ok(AppState { conn })
}

pub fn build_app(state: AppState) -> anyhow::Result<Router> {
    fs::create_dir_all(GLOBAL_CONFIG.avatar_path.clone())?;

    Ok(Router::new().nest("/api", main_router()).with_state(state))
}
