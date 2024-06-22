#![allow(clippy::never_loop)] // False positive

#[cfg(not(feature = "offline"))]
fn main() {
    if cfg!(feature = "offline") {
        panic!("This no binary is needed in web mode.");
    } else {
        panic!("Either the feature flag 'offline', or 'web_app' with wasm32 compilation must be enabled (offline is default).");
    }
}

#[cfg(feature = "offline")]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 1024.0])
            .with_drag_and_drop(true),

        ..Default::default()
    };

    eframe::run_native(
        "machete",
        options,
        Box::new(|cc| Box::new(machete_app::MainApp::new(cc))),
    )
}
