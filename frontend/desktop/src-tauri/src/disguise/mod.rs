//! Disguise mode: presents the app under another application's name.
//!
//! Name handling lives in [`name`], persistence in [`store`], and the Windows
//! process-name helpers in [`process_name`]; all three are covered by tests.
//! This module is the adapter that reaches into the Tauri `AppHandle` for the
//! app data directory, the main window, and the tray, so it is excluded from
//! coverage: see the coverage policy in `README.md`.

#[cfg(target_os = "windows")]
mod enumerate;
mod name;
#[cfg(target_os = "windows")]
mod process_name;
mod store;

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;
use name::sanitize_name;

pub const DEFAULT_APP_NAME: &str = "insomniAPP";
pub const TRAY_ID: &str = "main-tray";
const DISGUISE_WINDOW_LABEL: &str = "disguise";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisguiseStatePayload {
    pub supported: bool,
    pub current_name: String,
    pub is_disguised: bool,
}

pub fn is_supported() -> bool {
    cfg!(target_os = "windows")
}

pub fn initialize(app: &AppHandle) {
    let persisted_name = if is_supported() {
        load_persisted_name(app)
    } else {
        None
    };

    set_runtime_name(app, persisted_name);
    apply_identity(app);
}

pub fn get_state(app: &AppHandle) -> DisguiseStatePayload {
    let current_name = current_app_name(app);

    DisguiseStatePayload {
        supported: is_supported(),
        is_disguised: current_name != DEFAULT_APP_NAME,
        current_name,
    }
}

pub fn apply_disguise(app: &AppHandle, name: String) -> Result<(), String> {
    if !is_supported() {
        return Err("Disguise mode is only supported on Windows in this version.".into());
    }

    let sanitized =
        sanitize_name(&name).ok_or_else(|| "Disguise name cannot be empty".to_string())?;
    persist_name(app, Some(&sanitized))?;
    set_runtime_name(app, Some(sanitized));
    apply_identity(app);
    Ok(())
}

pub fn reset_disguise(app: &AppHandle) -> Result<(), String> {
    persist_name(app, None)?;
    set_runtime_name(app, None);
    apply_identity(app);
    Ok(())
}

pub fn clear_disguise_on_quit(app: &AppHandle) {
    let _ = persist_name(app, None);
    set_runtime_name(app, None);
}

pub fn open_disguise_window(app: &AppHandle) -> Result<(), String> {
    if !is_supported() {
        return Ok(());
    }

    let window = app
        .get_webview_window(DISGUISE_WINDOW_LABEL)
        .ok_or_else(|| "disguise window not found".to_string())?;

    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;

    let _ = window.emit("refresh-apps", ());

    Ok(())
}

pub fn list_running_apps() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        enumerate::list_windows_apps()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

// --- Internal helpers ---

fn current_app_name(app: &AppHandle) -> String {
    let state = app.state::<AppState>();
    let guard = state.disguise_name.lock().unwrap();
    guard
        .clone()
        .unwrap_or_else(|| DEFAULT_APP_NAME.to_string())
}

fn set_runtime_name(app: &AppHandle, name: Option<String>) {
    let state = app.state::<AppState>();
    let mut guard = state.disguise_name.lock().unwrap();
    *guard = name;
}

fn apply_identity(app: &AppHandle) {
    let name = current_app_name(app);

    #[cfg(target_os = "windows")]
    set_process_app_user_model_id(&name);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(&name);
    }

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(name));
    }
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map_err(|e| format!("failed to resolve app data directory: {e}"))
}

fn load_persisted_name(app: &AppHandle) -> Option<String> {
    store::load_name(&app_data_dir(app).ok()?)
}

fn persist_name(app: &AppHandle, name: Option<&str>) -> Result<(), String> {
    store::save_name(&app_data_dir(app)?, name)
}

#[cfg(target_os = "windows")]
fn set_process_app_user_model_id(name: &str) {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let app_id = name::app_user_model_id(name);
    let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(app_id)) };
}
