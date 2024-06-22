use egui::{containers::*, *};
use machete::models::campaign::Character;

/// Display a simple summary of the party members.
// TODO: We may want to add fields here (fields on the GUI itself)
#[derive(Default)]
pub struct PartyDisplay;

impl PartyDisplay {
    pub fn ui(&mut self, ui: &mut Ui, party: &[Character]) {
        ui.label("Party members:");

        ui.horizontal(|ui| {
            // TODO: Don't clone here- should use an arena to store all data.
            for (i, player) in party.iter().enumerate() {
                self.display_player(ui, player);

                if i < party.len() - 1 {
                    ui.separator();
                }
            }
        });
    }

    fn display_player(&mut self, ui: &mut Ui, player: &Character) {
        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(200.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(&player.name)
                        .text_style(TextStyle::Heading)
                        .strong(),
                );
                ui.label(&player.player);
            });
        });
    }
}
