use macros::IntoDataResponse;
use serde::Serialize;

#[derive(IntoDataResponse, Debug, Serialize)]
pub struct FileVersionResponse {
    pub version: String,
}
