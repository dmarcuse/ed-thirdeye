use serde::{Deserialize, Serialize};

use super::{PaneContext, TEPane};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NoStorage {}

#[typetag::serde]
impl TEPane for NoStorage {
    fn default_tab_name(&self) -> String {
        "Load error".into()
    }

    fn render(&mut self, _ctx: PaneContext<'_>, ui: &mut eframe::egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Data couldn't be loaded");
            ui.label(concat!(
                "Your previous settings and layout (if any) could not be loaded. ",
                "The program will still work, but you'll need to reapply your settings. ",
                "Please report this as a bug."
            ));
        });
    }
}
