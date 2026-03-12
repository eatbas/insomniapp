#[cfg(target_os = "windows")]
use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

#[cfg(target_os = "windows")]
pub fn list_windows_apps() -> Vec<String> {
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
fn to_wide_null(value: &str) -> Vec<u16> {
    OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
