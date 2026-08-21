//! Where the knowledge database and its backups live.
//!
//! Nothing in the codebase hard-codes a filesystem path: callers either pass an
//! explicit path (tests, `--db`) or ask for the platform data directory.

use std::path::PathBuf;

pub const APP_DIR_NAME: &str = "PersonalWiki";
pub const DB_FILE_NAME: &str = "personal-wiki.db";
pub const ENV_DB_PATH: &str = "PERSONAL_WIKI_DB";

/// Platform data directory: `%APPDATA%\PersonalWiki` on Windows,
/// `$XDG_DATA_HOME/PersonalWiki` (or `~/.local/share/PersonalWiki`) elsewhere.
pub fn data_dir() -> PathBuf {
    if let Ok(explicit) = std::env::var("PERSONAL_WIKI_HOME") {
        if !explicit.is_empty() {
            return PathBuf::from(explicit);
        }
    }
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join(APP_DIR_NAME);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join(APP_DIR_NAME);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share").join(APP_DIR_NAME);
    }
    PathBuf::from(APP_DIR_NAME)
}

/// The database path, honouring `PERSONAL_WIKI_DB` so tests and the CLI can point
/// at a scratch database without touching the user's data.
pub fn default_db_path() -> PathBuf {
    if let Ok(explicit) = std::env::var(ENV_DB_PATH) {
        if !explicit.is_empty() {
            return PathBuf::from(explicit);
        }
    }
    data_dir().join(DB_FILE_NAME)
}

pub fn backups_dir() -> PathBuf {
    default_db_path()
        .parent()
        .map(|p| p.join("backups"))
        .unwrap_or_else(|| data_dir().join("backups"))
}
