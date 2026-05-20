use crate::{
    AppState, dtos::files::FileVersionResponse, error::AppError, services::files::FilesService,
};
use axum::{Router, routing::get};

pub fn files_router() -> Router<AppState> {
    Router::new().route("/version", get(get_file_version))
}

async fn get_file_version() -> Result<FileVersionResponse, AppError> {
    let version = FilesService::get_version()?;
    Ok(FileVersionResponse { version })
}
