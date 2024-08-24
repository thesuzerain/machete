use crate::{
    apps::{library::LibraryApp, logbook::LogbookApp, summary::SummaryApp},
    settings_panel::SettingsPanel,
    update_context::UpdateWithContext,
};
use machete::models::{campaign::Campaign, library::Library};

pub struct MainApp {
    state: State,
}

/// Application-level GUI + context state.
pub struct State {
    // GUI state:
    summary: SummaryApp,
    logbook: LogbookApp,
    library: LibraryApp,
    settings_panel: SettingsPanel,
    selected_anchor: Anchor,

    // Application state:
    context: StateContext,
}

pub struct StateContext {
    pub campaign: Campaign,
    pub library: Library,
}

impl MainApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        // TODO: Allow storing state into file, onto web, etc.
        // This currently just loads a test fixture.
        let fixture = include_str!("../../../fixtures/demo_campaign.json");
        let campaign: Campaign =
            serde_json::from_str(fixture).expect("Failed to load test fixture.");

        let fixture = include_str!("../../../fixtures/demo_library.json");
        let library: Library = serde_json::from_str(fixture).expect("Failed to load test fixture.");

        #[allow(unused_mut)]
        let mut slf = Self {
            state: State {
                logbook: LogbookApp::start(&campaign),
                library: LibraryApp::start(),
                context: StateContext { campaign, library },
                summary: SummaryApp::default(),
                settings_panel: SettingsPanel::default(),
                selected_anchor: Anchor::default(),
            },
        };
        slf
    }

    /// Get an iterator over all the apps that can be shown.
    /// The return type is a tuple of:
    /// (iterator over (app_name, anchor, app), campaign)
    /// As the app requires a mutable reference to the StateContext, we pass it alongside the iterator instead of within it.
    fn apps_iter_mut(
        &mut self,
    ) -> (
        impl Iterator<Item = (&str, Anchor, &mut dyn UpdateWithContext)>,
        &mut StateContext,
    ) {
        let vec = vec![
            (
                "Summary",
                Anchor::Summary,
                &mut self.state.summary as &mut dyn UpdateWithContext,
            ),
            (
                "Logbook",
                Anchor::Logbook,
                &mut self.state.logbook as &mut dyn UpdateWithContext,
            ),
            (
                "Library",
                Anchor::Library,
                &mut self.state.library as &mut dyn UpdateWithContext,
            ),
        ];
        (vec.into_iter(), &mut self.state.context)
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_anchor = self.state.selected_anchor;
        let (apps_iter, state_context) = self.apps_iter_mut();
        for (_name, anchor, app) in apps_iter {
            if anchor == selected_anchor || ctx.memory(|mem| mem.everything_is_visible()) {
                app.update(ctx, frame, state_context);
            }
        }
    }

    /// Display the top bar with the app selection and settings.
    fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        ui.toggle_value(&mut self.state.settings_panel.open, "💻 Settings");

        ui.separator();

        let mut selected_anchor = self.state.selected_anchor;
        let (apps_iter, _) = self.apps_iter_mut();
        for (name, anchor, _app) in apps_iter {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
                if frame.is_web() {
                    ui.ctx()
                        .open_url(egui::OpenUrl::same_tab(format!("#{anchor}")));
                }
            }
        }
        self.state.selected_anchor = selected_anchor;

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::warn_if_debug_build(ui);
        });
    }

    /// Display the settings panel.
    fn settings_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // The settings-panel can be toggled on/off.
        // We show a little animation when the user switches it.
        let is_open =
            self.state.settings_panel.open || ctx.memory(|mem| mem.everything_is_visible());

        egui::SidePanel::left("settings_panel")
            .resizable(false)
            .show_animated(ctx, is_open, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("💻 Settings");
                });

                ui.separator();
                self.state.settings_panel.ui(ui, frame);
            });
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("wrap_app_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                self.bar_contents(ui, frame);
            });
        });

        self.settings_panel(ctx, frame);

        self.show_selected_app(ctx, frame);
        self.state.settings_panel.end_of_frame(ctx);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Anchor {
    Summary,
    Logbook,
    Library,
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<Anchor> for egui::WidgetText {
    fn from(value: Anchor) -> Self {
        Self::RichText(egui::RichText::new(value.to_string()))
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::Summary
    }
}
