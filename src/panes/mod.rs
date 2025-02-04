//! Implementations of individual panes that users can add to Third Eye

use eframe::egui::Ui;

use crate::settings::Settings;

mod about;
mod nostorage;
mod welcome;

pub use about::About;
pub use nostorage::NoStorage;
pub use welcome::Welcome;

/// Shared application state that panes can access
pub struct PaneContext<'a> {
    pub settings: &'a mut Settings,
}

/// A type of pane that users can add to Third Eye
#[typetag::serde]
pub trait TEPane {
    /// Get the default name to be used for the tab containing this pane
    fn default_tab_name(&self) -> String;

    /// Render this pane to the given UI
    fn render(&mut self, ctx: PaneContext<'_>, ui: &mut Ui);
}
