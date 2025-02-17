use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use eframe::{
    egui::{self, Modal, Ui, ViewportBuilder},
    NativeOptions,
};
use egui_tiles::{Container, SimplificationOptions, Tile, TileId, Tiles, Tree, UiResponse};
use log::{debug, info, warn};
use settings::{Settings, SettingsEditor};

use crate::panes::{PaneContext, TEPane, Welcome};

mod persistence;
pub mod settings;

#[derive(Debug)]
pub enum Message {
    AutoSave,
    AddPane {
        parent: TileId,
        pane: Box<dyn TEPane + Send>,
    },
    CloseSettingsModal {
        new_settings: Option<Settings>,
    },
}

/// Application layout and logic
struct App {
    data_dir: PathBuf,
    settings: Settings,
    layout: Tree<Box<dyn TEPane>>,
    message_tx: Sender<Message>,
    message_rx: Receiver<Message>,
    settings_editor: Option<SettingsEditor>,
}

impl App {
    const SETTINGS_FILE: &str = "settings.ron";
    const LAYOUT_FILE: &str = "layout.ron";

    /// Initialize the application, loading persistent state and layout data
    /// from eframe storage if possible
    fn init(data_dir: PathBuf, cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.all_styles_mut(|style| {
            style.interaction.selectable_labels = false;
        });

        let settings = match persistence::load_data(&data_dir.join(Self::SETTINGS_FILE)) {
            Ok(maybe_settings) => maybe_settings.unwrap_or_default(),
            Err(err) => {
                warn!("error loading saved settings: {err:?}");
                Settings::default()
            }
        };

        let layout = match persistence::load_data(&data_dir.join(Self::LAYOUT_FILE)) {
            Ok(Some(layout)) => layout,
            Ok(None) => Tree::new_tabs(
                "root_tabs",
                vec![Box::new(Welcome::default()) as Box<dyn TEPane>],
            ),
            Err(err) => {
                warn!("error loading saved layout: {err:?}");
                Tree::new_tabs(
                    "root_tabs",
                    vec![Box::new(Welcome::default()) as Box<dyn TEPane>],
                )
            }
        };

        let (message_tx, message_rx) = std::sync::mpsc::channel();

        {
            let ctx = cc.egui_ctx.clone();
            let message_tx = message_tx.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(30));
                match message_tx.send(Message::AutoSave) {
                    Ok(_) => ctx.request_repaint(),
                    Err(_) => break,
                }
            });
        }

        let app = App {
            data_dir,
            settings,
            layout,
            message_tx,
            message_rx,
            settings_editor: None,
        };
        app.apply_settings(&cc.egui_ctx);
        app
    }

    /// Save the persistent application state
    fn save_data(&self) {
        info!("saving persistent data...");

        if let Err(err) =
            persistence::save_data(&self.settings, &self.data_dir.join(Self::SETTINGS_FILE))
        {
            warn!("error saving settings: {err:?}");
        }

        if let Err(err) =
            persistence::save_data(&self.layout, &self.data_dir.join(Self::LAYOUT_FILE))
        {
            warn!("error saving layout: {err:?}");
        }
    }

    fn apply_settings(&self, ctx: &egui::Context) {
        ctx.request_repaint();
        ctx.set_theme(self.settings.theme);
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
        for message in self.message_rx.try_iter() {
            debug!("processing message: {message:#?}");
            match message {
                Message::AutoSave => self.save_data(),
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
                        self.settings = new_settings;
                        self.apply_settings(ctx);
                    }
                }
            }
        }
    }
}

struct AppBehavior<'a> {
    settings: &'a mut Settings,
    message_tx: &'a Sender<Message>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_global_hotkeys(ctx);
        self.avoid_empty_layout();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            if ui.button("Settings").clicked() {
                self.settings_editor = Some(self.settings.clone().into());
            }
        });

        let mut behavior = AppBehavior {
            settings: &mut self.settings,
            message_tx: &self.message_tx,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            self.layout.ui(&mut behavior, ui);
        });

        if let Some(settings_editor) = &mut self.settings_editor {
            let modal = Modal::new("settings".into());
            let response = modal.show(ctx, |ui| settings_editor.ui(ui));
            if let Some(msg) = response.inner {
                self.message_tx.send(msg).unwrap();
            }
        }

        self.handle_messages(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_data();
    }
}

/// Start the main graphical interface for the program
pub fn start(data_dir: PathBuf) -> eframe::Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_app_id(env!("CARGO_BIN_NAME")),
        ..Default::default()
    };
    eframe::run_native(
        "Third Eye",
        options,
        Box::new(|cc| Ok(Box::new(App::init(data_dir, cc)))),
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
            settings: &mut self.settings,
            message_tx: &self.message_tx,
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
                self.message_tx
                    .send(Message::AddPane {
                        parent: tile_id,
                        pane,
                    })
                    .unwrap();
            }
        });
    }
}
