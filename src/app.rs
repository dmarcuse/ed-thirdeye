use std::path::PathBuf;

use eframe::{
    egui::{self, Ui, ViewportBuilder},
    NativeOptions,
};
use egui_tiles::{Tree, UiResponse};
use serde::{Deserialize, Serialize};

use crate::{
    panes::{PaneContext, TEPane},
    settings::Settings,
};

/// Core application state for Third Eye
#[derive(Default, Serialize, Deserialize)]
struct PersistentState {
    settings: Settings,
}

/// Application layout and logic
struct App {
    state: PersistentState,
    layout: Tree<Box<dyn TEPane>>,
}

impl App {
    // key names for data stored in eframe's storage
    const STATE_KEY: &str = "thirdeye_state";
    const LAYOUT_KEY: &str = "thirdeye_layout";

    /// Initialize the application, loading persistent state and layout data
    /// from eframe storage if possible
    fn init(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        cc.egui_ctx
            .all_styles_mut(|style| style.interaction.selectable_labels = false);

        let state;
        let layout: Tree<Box<dyn TEPane>>;

        if let Some(storage) = cc.storage {
            state = eframe::get_value(storage, Self::STATE_KEY).unwrap_or_default();
            layout = eframe::get_value(storage, Self::LAYOUT_KEY).unwrap_or_else(|| {
                Tree::new_tabs(
                    Self::LAYOUT_KEY,
                    vec![Box::new(crate::panes::Welcome {}) as Box<dyn TEPane>],
                )
            });
        } else {
            state = PersistentState::default();
            layout = Tree::new_tabs(Self::LAYOUT_KEY, vec![Box::new(crate::panes::NoStorage {})]);
        }

        App { state, layout }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Self::STATE_KEY, &self.state);
        eframe::set_value(storage, Self::LAYOUT_KEY, &self.layout);
    }

    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.layout.ui(&mut self.state, ui);
        });
    }
}

/// Start the main graphical interface for the program
pub fn start(data_dir: PathBuf) -> eframe::Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_app_id(env!("CARGO_BIN_NAME")),
        persistence_path: Some(data_dir.join("state.ron")),
        ..Default::default()
    };
    eframe::run_native(
        "Third Eye",
        options,
        Box::new(|cc| Ok(Box::new(App::init(cc)))),
    )
}

impl egui_tiles::Behavior<Box<dyn TEPane>> for PersistentState {
    fn tab_title_for_pane(&mut self, pane: &Box<dyn TEPane>) -> egui::WidgetText {
        // TODO: allow the user to rename tabs
        pane.default_tab_name().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Box<dyn TEPane>,
    ) -> UiResponse {
        let context = PaneContext {
            settings: &mut self.settings,
        };
        pane.render(context, ui);
        UiResponse::None
    }
}
