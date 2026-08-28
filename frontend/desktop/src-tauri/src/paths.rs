//! Filesystem locations the app persists state to.
//!
//! Resolving the directory needs a live Tauri `AppHandle`, so this file is an
//! adapter with no decision of its own and is excluded from coverage: see the
//! coverage policy in `README.md`. The modules that persist state — the
//! disguise name and the keep-awake settings — keep their own document handling
//! in covered files and only borrow the directory from here.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

/// Resolves the app's local data directory, where every persisted state file
/// lives.
pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map_err(|e| format!("failed to resolve app data directory: {e}"))
}
