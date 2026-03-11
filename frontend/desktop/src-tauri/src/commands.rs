use tauri::{AppHandle, State};

use crate::disguise::{self, DisguiseStatePayload};
use crate::state::{AppState, AppStatus};

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> AppStatus {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
pub fn toggle_enabled(state: State<'_, AppState>) -> AppStatus {
    let mut status = state.status.lock().unwrap();
    status.enabled = !status.enabled;
    status.clone()
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPayload {
    pub idle_threshold_secs: Option<u64>,
    pub simulation_interval_secs: Option<u64>,
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: SettingsPayload) -> AppStatus {
    let mut status = state.status.lock().unwrap();
    if let Some(v) = settings.idle_threshold_secs {
        status.idle_threshold_secs = v;
    }
    if let Some(v) = settings.simulation_interval_secs {
        status.simulation_interval_secs = v;
    }
    status.clone()
}

#[tauri::command]
pub fn open_disguise_window(app: AppHandle) -> Result<(), String> {
    disguise::open_disguise_window(&app)
}

#[tauri::command]
pub fn get_disguise_state(app: AppHandle) -> DisguiseStatePayload {
    disguise::get_state(&app)
}

#[tauri::command]
pub fn list_running_apps() -> Vec<String> {
    disguise::list_running_apps()
}

#[tauri::command]
pub fn apply_disguise(app: AppHandle, name: String) -> Result<(), String> {
    disguise::apply_disguise(&app, name)
}

#[tauri::command]
pub fn reset_disguise(app: AppHandle) -> Result<(), String> {
    disguise::reset_disguise(&app)
}

#[tauri::command]
pub fn debug_log(message: String) {
    eprintln!("[ui-debug] {message}");
}
