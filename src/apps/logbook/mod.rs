use event_group_creator::EventGroupCreator;

use self::log::LogDisplay;
use crate::{models::campaign::Campaign, update_context::UpdateWithContext};

pub mod event_group_creator;
pub mod log;

/// Logbook application, for viewing and managing the event log.
pub struct LogbookApp {
    party_display: LogDisplay,
    event_creator: event_group_creator::EventGroupCreator,
}

impl LogbookApp {
    pub fn start(campaign: &Campaign) -> Self {
        LogbookApp {
            party_display: LogDisplay::start(&campaign.log),
            event_creator: EventGroupCreator::start(campaign),
        }
    }
}

impl UpdateWithContext for LogbookApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, campaign: &mut Campaign) {
        egui::TopBottomPanel::top("Logbook").show(ctx, |ui| {
            self.party_display.ui(ui, campaign);
        });

        egui::TopBottomPanel::top("Event Creator").show(ctx, |ui| {
            self.event_creator.ui(ui, campaign);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()))
            .show(ctx, |_ui| {
                // TODO: More panels in the summary app.
            });
    }
}