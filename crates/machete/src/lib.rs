use thiserror::Error;

// TODO: move
pub mod database;
pub mod models;

pub type Result<T> = std::result::Result<T, MacheteError>;

#[derive(Debug, Error)]
pub enum MacheteError {
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("SQLx error: {0}")]
    SqlxError(#[from] sqlx::Error),
}
