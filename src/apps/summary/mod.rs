use self::party_display::PartyDisplay;
use crate::{models::campaign::Campaign, update_context::UpdateWithContext};

pub mod party_display;

/// Summary of the campaign, including party members, current location, etc.
#[derive(Default)]
pub struct SummaryApp {
    party_display: PartyDisplay,
}

impl UpdateWithContext for SummaryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, campaign: &mut Campaign) {
        egui::TopBottomPanel::top("Summary").show(ctx, |ui| {
            self.party_display.ui(ui, &campaign.party);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()))
            .show(ctx, |_ui| {
                // TODO: More panels in the summary app.
            });
    }
}
