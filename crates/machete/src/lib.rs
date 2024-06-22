pub mod app;
pub use app::MainApp;

pub mod apps;
pub mod models;
pub mod settings_panel;
pub mod ui_models;
pub mod update_context;
pub mod utils;
pub mod widgets;

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
mod web;
#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
pub use web::*;
