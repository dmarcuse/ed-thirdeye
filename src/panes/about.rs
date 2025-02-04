use serde::{Deserialize, Serialize};

use super::{PaneContext, TEPane};

#[derive(Serialize, Deserialize)]
pub struct About {}

#[typetag::serde]
impl TEPane for About {
    fn default_tab_name(&self) -> String {
        "About".into()
    }

    fn render(&mut self, _ctx: PaneContext<'_>, ui: &mut eframe::egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Third Eye");
            ui.label(match cfg!(debug_assertions) {
                false => concat!("Version ", env!("CARGO_PKG_VERSION")),
                true => concat!("Version ", env!("CARGO_PKG_VERSION"), " [debug]"),
            });
            // TODO: also list authors, licenses, etc
        });
    }
}
