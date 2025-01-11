use thiserror::Error;

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
    #[error("SS14 Auth Server not found error.")]
    AuthNotFound
}