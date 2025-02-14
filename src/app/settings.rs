use std::path::PathBuf;

use eframe::egui::{Grid, TextBuffer, ThemePreference, Ui};
use serde::{Deserialize, Serialize};

use super::Message;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalPath {
    String(String),
    Path(PathBuf),
    Unset,
}

impl Default for JournalPath {
    fn default() -> Self {
        let suffix = PathBuf::new()
            .join("Saved Games")
            .join("Frontier Developments")
            .join("Elite Dangerous");

        if cfg!(target_os = "windows") {
            dirs::home_dir()
                .map(|p| p.join(suffix))
                .map(Self::Path)
                .unwrap_or(Self::Unset)
        } else if cfg!(target_os = "linux") {
            // assume that the game is running in Steam via Proton
            dirs::data_dir()
                .map(|p| {
                    p.join("Steam")
                        .join("steamapps")
                        .join("compatdata")
                        .join("359320") // Elite: Dangerous steam app ID
                        .join("pfx")
                        .join("drive_c")
                        .join("users")
                        .join("steamuser")
                        .join(suffix)
                })
                .map(Self::Path)
                .unwrap_or(Self::Unset)
        } else {
            Self::Unset
        }
    }
}

impl JournalPath {
    /// Create an editable TextBuffer for this journal path, which will lossily
    /// convert the underlying data to a String if necessary.
    fn as_text_buffer(&mut self) -> impl TextBuffer + use<'_> {
        struct JournalPathTextBuffer<'a> {
            string_repr: String,
            inner: &'a mut JournalPath,
        }

        impl TextBuffer for JournalPathTextBuffer<'_> {
            fn is_mutable(&self) -> bool {
                true
            }

            fn as_str(&self) -> &str {
                &self.string_repr
            }

            fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
                let inserted_count = self.string_repr.insert_text(text, char_index);
                *self.inner = JournalPath::String(self.string_repr.clone());
                inserted_count
            }

            fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
                self.string_repr.delete_char_range(char_range);
                *self.inner = JournalPath::String(self.string_repr.clone());
            }
        }

        let string_repr = match self {
            Self::String(s) => s.to_owned(),
            Self::Unset => String::new(),
            Self::Path(p) => p.to_string_lossy().into_owned(),
        };

        JournalPathTextBuffer {
            string_repr,
            inner: self,
        }
    }
}

/// Persistent user settings for Third Eye
///
/// These settings should be backwards compatible, such that settings saved by
/// older versions of the program can be loaded in newer versions to avoid
/// annoying the user by resetting their configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The egui theme to use
    pub theme: ThemePreference,

    /// The path to Elite: Dangerous journal files
    pub journal_path: JournalPath,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            journal_path: JournalPath::default(),
        }
    }
}

/// State associated with the settings modal
#[derive(Debug)]
pub struct SettingsEditor {
    settings: Settings,
}

impl From<Settings> for SettingsEditor {
    fn from(settings: Settings) -> Self {
        Self { settings }
    }
}

impl SettingsEditor {
    pub fn ui(&mut self, ui: &mut Ui) -> Option<Message> {
        ui.vertical(|ui| {
            ui.heading("Settings");
            Grid::new("settings_grid").num_columns(2).show(ui, |ui| {
                ui.label("Theme");
                self.settings.theme.radio_buttons(ui);
                ui.end_row();

                ui.label("Journal folder");
                ui.text_edit_singleline(&mut self.settings.journal_path.as_text_buffer());
                ui.end_row();
            });
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    return Some(Message::CloseSettingsModal { new_settings: None });
                }
                if ui.button("Save").clicked() {
                    return Some(Message::CloseSettingsModal {
                        new_settings: Some(self.settings.clone()),
                    });
                }

                None
            })
            .inner
        })
        .inner
    }
}
