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

/// Render the menu for adding a new tab
pub fn new_pane_menu_ui(ui: &mut Ui) -> Option<Box<dyn TEPane>> {
    const fn ctor<T: 'static + TEPane + Default>() -> fn() -> Box<dyn TEPane> {
        || Box::new(T::default())
    }
    static USER_CREATABLE_PANES: &[(&str, fn() -> Box<dyn TEPane>)] =
        &[("Welcome", ctor::<Welcome>()), ("About", ctor::<About>())];

    for &(name, ctor) in USER_CREATABLE_PANES {
        if ui.button(name).clicked() {
            ui.close_menu();
            return Some(ctor());
        }
    }

    None
}
