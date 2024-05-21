#![allow(clippy::never_loop)] // False positive

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
        Box::new(|cc| Box::new(machete::MainApp::new(cc))),
    )
}
