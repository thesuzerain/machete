use thiserror::Error;

pub mod app;
pub use app::MainApp;

pub mod apps;
pub mod fetch;
pub mod settings_panel;
pub mod ui_models;
pub mod update_context;
pub mod utils;
pub mod widgets;

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
mod web;
#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
pub use web::*;

pub type Result<T> = std::result::Result<T, MacheteError>;

#[derive(Debug, Error)]
pub enum MacheteError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Communication error: {0}")]
    CommunicationError(#[from] reqwest::Error),
}
