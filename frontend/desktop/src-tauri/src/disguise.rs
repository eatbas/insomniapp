use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Emitter, Manager};

pub const DEFAULT_APP_NAME: &str = "insomniAPP";
pub const TRAY_ID: &str = "main-tray";
const DISGUISE_STATE_FILE: &str = "disguise_state.json";
const DISGUISE_WINDOW_LABEL: &str = "disguise";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisguiseStatePayload {
    pub supported: bool,
    pub current_name: String,
    pub is_disguised: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedDisguiseState {
    pub name: Option<String>,
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
    eprintln!(
        "[disguise] get_state supported={} current_name={}",
        is_supported(),
        current_name
    );

    DisguiseStatePayload {
        supported: is_supported(),
        is_disguised: current_name != DEFAULT_APP_NAME,
        current_name,
    }
}

pub fn apply_disguise(app: &AppHandle, name: String) -> Result<(), String> {
    eprintln!("[disguise] apply_disguise requested name={name}");

    if !is_supported() {
        return Err("Disguise mode is only supported on Windows in this version.".into());
    }

    let sanitized =
        sanitize_name(&name).ok_or_else(|| "Disguise name cannot be empty".to_string())?;
    persist_name(app, Some(sanitized.clone()))?;
    set_runtime_name(app, Some(sanitized));
    apply_identity(app);
    Ok(())
}

pub fn reset_disguise(app: &AppHandle) -> Result<(), String> {
    eprintln!("[disguise] reset_disguise");
    persist_name(app, None)?;
    set_runtime_name(app, None);
    apply_identity(app);
    Ok(())
}

pub fn clear_disguise_on_quit(app: &AppHandle) {
    eprintln!("[disguise] clear_disguise_on_quit");
    let _ = persist_name(app, None);
    set_runtime_name(app, None);
}

pub fn open_disguise_window(app: &AppHandle) -> Result<(), String> {
    eprintln!(
        "[disguise] open_disguise_window supported={}",
        is_supported()
    );

    if !is_supported() {
        return Ok(());
    }

    let window = app
        .get_webview_window(DISGUISE_WINDOW_LABEL)
        .ok_or_else(|| "disguise window not found".to_string())?;

    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;

    // Tell the frontend to refresh the app list
    let _ = window.emit("refresh-apps", ());

    eprintln!("[disguise] disguise window shown");
    Ok(())
}

pub fn list_running_apps() -> Vec<String> {
    eprintln!("[disguise] list_running_apps called");
    #[cfg(target_os = "windows")]
    {
        list_windows_apps()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

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
    eprintln!("[disguise] apply_identity name={name}");

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_title(&name);
    }

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(name));
    }
}

fn sanitize_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(80).collect())
    }
}

fn load_persisted_name(app: &AppHandle) -> Option<String> {
    let dir = app.path().app_local_data_dir().ok()?;
    let path = dir.join(DISGUISE_STATE_FILE);
    let content = fs::read_to_string(path).ok()?;
    let persisted = serde_json::from_str::<PersistedDisguiseState>(&content).ok()?;
    persisted.name.and_then(|name| sanitize_name(&name))
}

fn persist_name(app: &AppHandle, name: Option<String>) -> Result<(), String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("failed to resolve app data directory: {e}"))?;

    fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create app data directory {}: {e}", dir.display()))?;

    let path = dir.join(DISGUISE_STATE_FILE);
    let payload = PersistedDisguiseState { name };
    let json = serde_json::to_string(&payload)
        .map_err(|e| format!("failed to serialize disguise state: {e}"))?;

    fs::write(&path, json)
        .map_err(|e| format!("failed to write disguise state to {}: {e}", path.display()))
}

#[cfg(target_os = "windows")]
fn list_windows_apps() -> Vec<String> {
    use std::collections::HashMap;
    use windows::core::{BOOL, PWSTR};
    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindow, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible,
        GW_OWNER,
    };

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let apps = &mut *(lparam.0 as *mut HashMap<String, String>);

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        if let Ok(owner) = GetWindow(hwnd, GW_OWNER) {
            if !owner.0.is_null() {
                return BOOL(1);
            }
        } else {
            return BOOL(1);
        }

        if GetWindowTextLengthW(hwnd) <= 0 {
            return BOOL(1);
        }

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 || pid == std::process::id() {
            return BOOL(1);
        }

        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return BOOL(1);
        };

        let mut buffer = vec![0u16; 260];
        let mut size = buffer.len() as u32;
        let path_result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);

        let Ok(()) = path_result else {
            return BOOL(1);
        };

        if size == 0 {
            return BOOL(1);
        }

        let exe_path = String::from_utf16_lossy(&buffer[..size as usize]);
        let Some(stem) = std::path::Path::new(&exe_path).file_stem() else {
            return BOOL(1);
        };

        let name = stem.to_string_lossy().trim().to_string();
        if name.is_empty() || is_noise_process(&name) {
            return BOOL(1);
        }

        apps.entry(name.to_ascii_lowercase()).or_insert(name);
        BOOL(1)
    }

    let mut apps = HashMap::<String, String>::new();

    unsafe {
        let apps_ptr = &mut apps as *mut HashMap<String, String>;
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(apps_ptr as isize));
    }

    let mut values: Vec<String> = apps.into_values().collect();
    values.sort_by_key(|name| name.to_ascii_lowercase());
    eprintln!("[disguise] list_windows_apps found {} apps", values.len());
    values
}

#[cfg(target_os = "windows")]
fn is_noise_process(name: &str) -> bool {
    [
        "ApplicationFrameHost",
        "ShellExperienceHost",
        "SearchHost",
        "StartMenuExperienceHost",
        "TextInputHost",
        "RuntimeBroker",
        "Widgets",
        "dwm",
        "sihost",
        "ctfmon",
        "taskhostw",
        "insomniapp",
    ]
    .iter()
    .any(|blocked| blocked.eq_ignore_ascii_case(name))
}
