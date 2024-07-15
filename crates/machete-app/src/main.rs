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
    // Tokio runtime
    // https://github.com/parasyte/egui-tokio-example/blob/main/src/main.rs

    use std::time::Duration;
    use tokio::runtime::Builder;

    let rt = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .thread_name("machete-tokio")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();

    let _enter = rt.enter();
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    // Egui runtime
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
