use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Emitter, Manager};
#[cfg(target_os = "windows")]
use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

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
    persist_name(app, Some(sanitized.clone()))?;
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

    // Tell the frontend to refresh the app list
    let _ = window.emit("refresh-apps", ());

    Ok(())
}

pub fn list_running_apps() -> Vec<String> {
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

    #[cfg(target_os = "windows")]
    set_process_app_user_model_id(&name);

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

        // `GetWindow(..., GW_OWNER)` returns a null HWND when there is no owner.
        // In `windows` crate that null can surface as `Err` (with last-error 0),
        // so treat errors here as "no owner" instead of dropping the window.
        let has_owner = match GetWindow(hwnd, GW_OWNER) {
            Ok(owner) => !owner.0.is_null(),
            Err(_) => false,
        };
        if has_owner {
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

        let exe_stem = stem.to_string_lossy().trim().to_string();
        if exe_stem.is_empty() || is_noise_process(&exe_stem) {
            return BOOL(1);
        }

        let display_name =
            friendly_process_name(&exe_path, &exe_stem).unwrap_or_else(|| exe_stem.clone());
        apps.entry(display_name.to_ascii_lowercase())
            .or_insert(display_name);
        BOOL(1)
    }

    let mut apps = HashMap::<String, String>::new();

    unsafe {
        let apps_ptr = &mut apps as *mut HashMap<String, String>;
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(apps_ptr as isize));
    }

    let mut values: Vec<String> = apps.into_values().collect();
    values.sort_by_key(|name| name.to_ascii_lowercase());
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

#[cfg(target_os = "windows")]
fn friendly_process_name(exe_path: &str, exe_stem: &str) -> Option<String> {
    file_version_value(exe_path, "FileDescription")
        .or_else(|| file_version_value(exe_path, "ProductName"))
        .or_else(|| prettify_stem(exe_stem))
}

#[cfg(target_os = "windows")]
fn file_version_value(exe_path: &str, key: &str) -> Option<String> {
    use std::{ffi::c_void, ptr};
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };

    let path_w = to_wide_null(exe_path);
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(path_w.as_ptr()), None) };
    if size == 0 {
        return None;
    }

    let mut data = vec![0u8; size as usize];
    if unsafe {
        GetFileVersionInfoW(
            PCWSTR(path_w.as_ptr()),
            None,
            size,
            data.as_mut_ptr() as *mut c_void,
        )
    }
    .is_err()
    {
        return None;
    }

    let mut translations = version_translations(&data);
    translations.push((0x0409, 0x04B0));
    translations.push((0x0000, 0x04B0));

    for (lang, codepage) in translations {
        let query = format!("\\StringFileInfo\\{lang:04x}{codepage:04x}\\{key}");
        let query_w = to_wide_null(&query);
        let mut value_ptr: *mut c_void = ptr::null_mut();
        let mut value_len = 0u32;
        let found = unsafe {
            VerQueryValueW(
                data.as_ptr() as *const c_void,
                PCWSTR(query_w.as_ptr()),
                &mut value_ptr,
                &mut value_len,
            )
            .as_bool()
        };

        if !found || value_ptr.is_null() || value_len == 0 {
            continue;
        }

        let value = unsafe {
            let slice = std::slice::from_raw_parts(value_ptr as *const u16, value_len as usize);
            String::from_utf16_lossy(slice)
        };
        let normalized = value.trim_matches('\0').trim();
        if !normalized.is_empty() {
            return Some(normalized.to_string());
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn version_translations(data: &[u8]) -> Vec<(u16, u16)> {
    use std::{ffi::c_void, ptr};
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::VerQueryValueW;

    let query_w = to_wide_null("\\VarFileInfo\\Translation");
    let mut trans_ptr: *mut c_void = ptr::null_mut();
    let mut trans_len = 0u32;

    let found = unsafe {
        VerQueryValueW(
            data.as_ptr() as *const c_void,
            PCWSTR(query_w.as_ptr()),
            &mut trans_ptr,
            &mut trans_len,
        )
        .as_bool()
    };

    if !found || trans_ptr.is_null() || trans_len < 4 {
        return Vec::new();
    }

    let count = (trans_len as usize) / 4;
    let words = unsafe { std::slice::from_raw_parts(trans_ptr as *const u16, count * 2) };

    let mut pairs = Vec::with_capacity(count);
    for chunk in words.chunks_exact(2) {
        pairs.push((chunk[0], chunk[1]));
    }

    pairs.sort_unstable();
    pairs.dedup();
    pairs
}

#[cfg(target_os = "windows")]
fn prettify_stem(stem: &str) -> Option<String> {
    let cleaned = stem.trim();
    if cleaned.is_empty() {
        return None;
    }

    let words: Vec<String> = cleaned
        .split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect();

    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

#[cfg(target_os = "windows")]
fn set_process_app_user_model_id(name: &str) {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let mut slug = String::new();
    let mut last_was_dot = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dot = false;
        } else if !last_was_dot {
            slug.push('.');
            last_was_dot = true;
        }
    }

    let slug = slug.trim_matches('.').to_string();
    let slug = if slug.is_empty() {
        "insomniapp".to_string()
    } else {
        slug
    };

    let app_id = format!("com.insomniapp.{slug}");
    let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(app_id)) };
}

#[cfg(target_os = "windows")]
fn to_wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
