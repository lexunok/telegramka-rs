use crate::{
    config::GLOBAL_CONFIG, handlers::main_router, utils::startup::create_admin,
    workers::invitation_worker,
};
use axum::Router;
use axum::http::{HeaderValue, Method, header};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::fs;
use tower_http::cors::CorsLayer;

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
    create_admin(state.conn.clone()).await.unwrap();

    let redis_clone = state.redis_client.clone();
    tokio::spawn(async move {
        invitation_worker::invitation_worker(redis_clone).await;
    });

    let app = build_app(state)?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", GLOBAL_CONFIG.port)).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    conn: DatabaseConnection,
    redis_client: redis::Client,
}

impl AppState {
    pub fn new(conn: DatabaseConnection, redis_client: redis::Client) -> Self {
        Self { conn, redis_client }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

pub async fn init_app_state() -> anyhow::Result<AppState> {
    let conn = Database::connect(GLOBAL_CONFIG.db_url.to_owned()).await?;
    Migrator::up(&conn, None).await?;
    let redis_client = redis::Client::open(GLOBAL_CONFIG.redis_url.to_owned())?;

    Ok(AppState::new(conn, redis_client))
}

pub fn build_app(state: AppState) -> anyhow::Result<Router> {
    let cors = CorsLayer::new()
        .allow_origin(
            GLOBAL_CONFIG
                .client_url
                .clone()
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_credentials(true)
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    fs::create_dir_all(GLOBAL_CONFIG.avatar_path.clone())?;

    Ok(Router::new()
        .nest("/api", main_router())
        .with_state(state)
        .layer(cors))
}
