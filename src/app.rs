use std::path::PathBuf;

use eframe::{
    egui::{self, Ui, ViewportBuilder},
    NativeOptions,
};
use egui_tiles::{Container, SimplificationOptions, Tile, TileId, Tiles, Tree, UiResponse};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    panes::{self, PaneContext, TEPane},
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
        let layout;

        if let Some(storage) = cc.storage {
            state = eframe::get_value(storage, Self::STATE_KEY).unwrap_or_default();
            layout = eframe::get_value(storage, Self::LAYOUT_KEY).unwrap_or_else(|| {
                Tree::new_tabs(
                    Self::LAYOUT_KEY,
                    vec![Box::new(panes::Welcome::default()) as Box<dyn TEPane>],
                )
            });
        } else {
            state = PersistentState::default();
            layout = Tree::new_tabs(
                Self::LAYOUT_KEY,
                vec![Box::new(panes::NoStorage::default())],
            );
        }

        App { state, layout }
    }
}

struct AppBehavior<'a> {
    persistent: &'a mut PersistentState,
    add_pane: Option<(TileId, Box<dyn TEPane>)>,
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Self::STATE_KEY, &self.state);
        eframe::set_value(storage, Self::LAYOUT_KEY, &self.layout);
    }

    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.layout.is_empty() {
            info!("layout contains no tiles - adding one...");
            let id = self
                .layout
                .tiles
                .insert_pane(Box::new(crate::panes::Welcome {}));
            self.layout.root = Some(id);
        }

        let mut behavior = AppBehavior {
            persistent: &mut self.state,
            add_pane: None,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            self.layout.ui(&mut behavior, ui);
        });

        if let Some((parent, pane)) = behavior.add_pane.take() {
            let child = self.layout.tiles.insert_pane(pane);
            if let Some(Tile::Container(parent)) = self.layout.tiles.get_mut(parent) {
                parent.add_child(child);
                if let Container::Tabs(tabs) = parent {
                    tabs.set_active(child);
                }
            }
        }
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

impl<'a> egui_tiles::Behavior<Box<dyn TEPane>> for AppBehavior<'a> {
    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn tab_title_for_pane(&mut self, pane: &Box<dyn TEPane>) -> egui::WidgetText {
        // TODO: allow the user to rename tabs, e.g. by right clicking them
        pane.default_tab_name().into()
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Box<dyn TEPane>>, _tile_id: TileId) -> bool {
        true
    }

    fn pane_ui(&mut self, ui: &mut Ui, _tile_id: TileId, pane: &mut Box<dyn TEPane>) -> UiResponse {
        let context = PaneContext {
            settings: &mut self.persistent.settings,
        };
        pane.render(context, ui);
        UiResponse::None
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &Tiles<Box<dyn TEPane>>,
        ui: &mut Ui,
        tile_id: TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        ui.menu_button("+", |ui| {
            self.add_pane = crate::panes::new_pane_menu_ui(ui).map(|pane| (tile_id, pane));
        });
    }
}
