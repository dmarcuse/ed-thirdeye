use serde::{Deserialize, Serialize};

use super::{PaneContext, TEPane};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Welcome {}

#[typetag::serde]
impl TEPane for Welcome {
    fn default_tab_name(&self) -> String {
        "Welcome".into()
    }

    fn render(&mut self, _ctx: PaneContext<'_>, ui: &mut eframe::egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Welcome to Third Eye!");
            ui.label("Your Elite: Dangerous exploration assistant");
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("This software is still in early development. Please ");
                ui.hyperlink_to("report any issues", env!("CARGO_PKG_REPOSITORY"));
                ui.label(" you encounter.");
            });
        });
    }
}
