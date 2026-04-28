use std::{fs, path::PathBuf};

use crate::{config::GLOBAL_CONFIG, error::AppError};

pub struct FilesService;

impl FilesService {
    pub fn get_version() -> Result<String, AppError> {
        let path = Self::find_single_file()?;
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or(AppError::BadRequest)?;

        if stem.is_empty() {
            return Err(AppError::BadRequest);
        }

        Ok(stem.to_string())
    }

    fn find_single_file() -> Result<PathBuf, AppError> {
        let mut files = fs::read_dir(&GLOBAL_CONFIG.release_path)
            .map_err(|_| AppError::InternalServerError)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file());

        let file = files.next().ok_or(AppError::NotFound)?;

        if files.next().is_some() {
            return Err(AppError::Custom(
                "Expected exactly one release file in RELEASE_PATH".to_string(),
            ));
        }

        Ok(file)
    }
}
