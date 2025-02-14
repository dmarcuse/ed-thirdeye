use std::path::PathBuf;

use eframe::{
    egui::{self, Modal, Ui, ViewportBuilder},
    NativeOptions,
};
use egui_tiles::{Container, SimplificationOptions, Tile, TileId, Tiles, Tree, UiResponse};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsEditor};

use crate::panes::{self, PaneContext, TEPane};

pub mod settings;

#[derive(Debug)]
pub enum Message {
    AddPane {
        parent: TileId,
        pane: Box<dyn TEPane>,
    },
    CloseSettingsModal {
        new_settings: Option<Settings>,
    },
}

/// Core application state for Third Eye
#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistentState {
    settings: Settings,
}

/// Application layout and logic
struct App {
    state: PersistentState,
    layout: Tree<Box<dyn TEPane>>,
    messages: Vec<Message>,
    settings_editor: Option<SettingsEditor>,
}

impl App {
    // key names for data stored in eframe's storage
    const STATE_KEY: &str = "thirdeye_state";
    const LAYOUT_KEY: &str = "thirdeye_layout";

    /// Initialize the application, loading persistent state and layout data
    /// from eframe storage if possible
    fn init(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        cc.egui_ctx.all_styles_mut(|style| {
            style.interaction.selectable_labels = false;
            // style.debug.show_expand_width = true;
            // style.debug.debug_on_hover = true;
        });

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

        App {
            state,
            layout,
            messages: Vec::new(),
            settings_editor: None,
        }
    }

    fn handle_global_hotkeys(&mut self, ctx: &egui::Context) {
        #[cfg(debug_assertions)]
        {
            let debug_toggled = ctx.input(|input| {
                input.events.iter().any(|e| {
                    matches!(
                        e,
                        egui::Event::Key {
                            key: egui::Key::F12,
                            pressed: true,
                            repeat: false,
                            ..
                        }
                    )
                })
            });

            if debug_toggled {
                ctx.style_mut(|style| style.debug.debug_on_hover = !style.debug.debug_on_hover);
            }
        }
    }

    /// Check whether the current layout is empty, and add a tile if so
    fn avoid_empty_layout(&mut self) {
        if self.layout.is_empty() {
            info!("layout contains no tiles - adding one...");
            let id = self
                .layout
                .tiles
                .insert_pane(Box::new(crate::panes::Welcome {}));
            self.layout.root = Some(id);
        }
    }

    /// Handle any new messages
    fn handle_messages(&mut self, ctx: &egui::Context) {
        for message in self.messages.drain(..) {
            debug!("processing message: {message:#?}");
            match message {
                Message::AddPane { parent, pane } => {
                    let child = self.layout.tiles.insert_pane(pane);
                    match self.layout.tiles.get_mut(parent) {
                        Some(Tile::Container(parent)) => {
                            parent.add_child(child);
                            if let Container::Tabs(tabs) = parent {
                                tabs.set_active(child);
                            }
                        }
                        parent => {
                            warn!("cannot open a new pane in non-container parent: {parent:#?}");
                        }
                    }
                }
                Message::CloseSettingsModal { new_settings } => {
                    self.settings_editor = None;
                    if let Some(new_settings) = new_settings {
                        self.state.settings = new_settings;
                        ctx.set_theme(self.state.settings.theme);
                    }
                }
            }
        }
    }
}

struct AppBehavior<'a> {
    persistent: &'a mut PersistentState,
    messages: &'a mut Vec<Message>,
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Self::STATE_KEY, &self.state);
        eframe::set_value(storage, Self::LAYOUT_KEY, &self.layout);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_global_hotkeys(ctx);
        self.avoid_empty_layout();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            if ui.button("Settings").clicked() {
                self.settings_editor = Some(self.state.settings.clone().into());
            }
        });

        let mut behavior = AppBehavior {
            persistent: &mut self.state,
            messages: &mut self.messages,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            self.layout.ui(&mut behavior, ui);
        });

        if let Some(settings_editor) = &mut self.settings_editor {
            let modal = Modal::new("settings".into());
            let response = modal.show(ctx, |ui| settings_editor.ui(ui));
            self.messages.extend(response.inner);
        }

        self.handle_messages(ctx);
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
        egui::Frame::new()
            .inner_margin(3)
            .show(ui, |ui| pane.render(context, ui));

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
            if let Some(pane) = crate::panes::new_pane_menu_ui(ui) {
                self.messages.push(Message::AddPane {
                    parent: tile_id,
                    pane,
                });
            }
        });
    }
}
