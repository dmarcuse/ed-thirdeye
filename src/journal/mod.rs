//! Locate, parse, and monitor Elite: Dangerous journal files

use std::path::PathBuf;

/// Get the default Elite: Dangerous journal file path for the current system.
///
/// Returns `None` when no default is known.
pub fn get_default_journal_path() -> Option<PathBuf> {
    let suffix = PathBuf::new()
        .join("Saved Games")
        .join("Frontier Developments")
        .join("Elite Dangerous");

    if cfg!(target_os = "windows") {
        dirs::home_dir().map(|p| p.join(suffix))
    } else if cfg!(target_os = "linux") {
        // assume that the game is running in Steam via Proton
        dirs::data_dir().map(|p| {
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
    } else {
        None
    }
}
