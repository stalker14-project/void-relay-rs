use thiserror::Error;

use crate::api::ss14client::ErrorResponse;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O Error: {0}")]
    IoErr(#[from] std::io::Error),
    #[error("Serialization Error {0}")]
    SerdeErr(#[from] serde_json::Error),
    #[error("SQLx Error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Serenity client error: {0}")]
    SerenityErr(#[from] serenity::Error),
    #[error("Reqwest error: {0}")]
    ReqwestErr(#[from] reqwest::Error),
    #[error("UUID Parse Error: {0}")]
    UuidErr(#[from] uuid::Error),
    #[error("SS14 Api Error: {0}")]
    SS14ApiError(String),
    #[error("TypeAuthD Api Error: {0}")]
    TypeAuthDApiError(String),
}

impl From<ErrorResponse> for Error {
    fn from(value: ErrorResponse) -> Self {
        Self::SS14ApiError(value.to_string())
    }
}

impl Error {
    pub fn ss14_api(err: &str) -> Self {
        Self::SS14ApiError(err.to_string())
    }
}