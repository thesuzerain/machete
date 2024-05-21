/// Hideable settings panel on the side.
/// Includes settings, account information, etc.
/// Currently, a lot of this is taken directly from the egui demo.
#[derive(Default)]
pub struct SettingsPanel {
    pub open: bool,

    settings_windows: SettingsWindows,
}

impl SettingsPanel {
    pub fn end_of_frame(&mut self, ctx: &egui::Context) {
        self.settings_windows.windows(ctx);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        integration_ui(ui, frame);

        ui.separator();

        ui.label("settings windows:");
        self.settings_windows.checkboxes(ui);

        #[cfg(debug_assertions)]
        if ui.ctx().style().debug.debug_on_hover_with_all_modifiers {
            ui.separator();
            ui.label("Press down all modifiers and hover a widget to see a callstack for it");
        }

        if cfg!(debug_assertions) && cfg!(target_arch = "wasm32") {
            ui.separator();
            // For testing panic handling on web:
            #[allow(clippy::manual_assert)]
            if ui.button("panic!()").clicked() {
                panic!("intentional panic!");
            }
        }

        if !cfg!(target_arch = "wasm32") {
            ui.separator();
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn integration_ui(ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("egui running inside ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });

    #[cfg(target_arch = "wasm32")]
    ui.collapsing("Web info (location)", |ui| {
        ui.style_mut().wrap = Some(false);
        ui.monospace(format!("{:#?}", _frame.info().web_info.location));
    });

    #[cfg(not(target_arch = "wasm32"))]
    {
        ui.horizontal(|ui| {
            {
                let mut fullscreen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
                if ui
                    .checkbox(&mut fullscreen, "🗖 Fullscreen (F11)")
                    .on_hover_text("Fullscreen the window")
                    .changed()
                {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Fullscreen(fullscreen));
                }
            }

            let mut size = None;
            egui::ComboBox::from_id_source("viewport-size-combo")
                .selected_text("Resize to...")
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut size,
                        Some(egui::vec2(375.0, 667.0)),
                        "📱 iPhone SE 2nd Gen",
                    );
                    ui.selectable_value(&mut size, Some(egui::vec2(393.0, 852.0)), "📱 iPhone 15");
                    ui.selectable_value(
                        &mut size,
                        Some(egui::vec2(1280.0, 720.0)),
                        "🖥 Desktop 720p",
                    );
                    ui.selectable_value(
                        &mut size,
                        Some(egui::vec2(1920.0, 1080.0)),
                        "🖥 Desktop 1080p",
                    );
                });

            if let Some(size) = size {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ui.close_menu();
            }
        });
    }
}

/// Wrapper around the egui settings display.
struct SettingsWindows {
    settings: bool,
}

impl Default for SettingsWindows {
    fn default() -> Self {
        Self::none()
    }
}

impl SettingsWindows {
    fn none() -> Self {
        Self { settings: false }
    }

    fn checkboxes(&mut self, ui: &mut egui::Ui) {
        let Self { settings } = self;

        ui.checkbox(settings, "🔧 Settings");
    }

    fn windows(&mut self, ctx: &egui::Context) {
        let Self { settings } = self;

        egui::Window::new("🔧 Settings")
            .open(settings)
            .vscroll(true)
            .show(ctx, |ui| {
                // TODO: The settings popup should not use the default egui settings.
                ctx.settings_ui(ui);
            });
    }
}
