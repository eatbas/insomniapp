//! Windows top-level window enumeration.
//!
//! Every function here wraps a Win32 call whose result depends on which windows
//! happen to be open, so this file is excluded from the coverage gate; the pure
//! logic it relies on lives in [`super::process_name`], which is covered. The
//! smoke tests below still drive the real `EnumWindows` traversal and the real
//! version-resource lookup. See the coverage policy in `README.md`.

use super::process_name::{is_noise_process, prettify_stem, to_wide_null, translation_pairs};

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

fn friendly_process_name(exe_path: &str, exe_stem: &str) -> Option<String> {
    file_version_value(exe_path, "FileDescription")
        .or_else(|| file_version_value(exe_path, "ProductName"))
        .or_else(|| prettify_stem(exe_stem))
}

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

    translation_pairs(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test over the real `EnumWindows` traversal. A CI host may have no
    /// visible top-level windows at all, so the assertions describe the shape of
    /// the result rather than its contents.
    #[test]
    fn list_windows_apps_returns_sorted_names_without_noise_or_self() {
        let apps = list_windows_apps();

        let mut expected = apps.clone();
        expected.sort_by_key(|name| name.to_ascii_lowercase());
        assert_eq!(apps, expected, "results must be sorted case-insensitively");

        assert!(
            !apps
                .iter()
                .any(|name| name.eq_ignore_ascii_case("insomniapp")),
            "the app must never offer itself as a disguise"
        );
        assert!(
            !apps.iter().any(|name| name.trim().is_empty()),
            "blank names must be filtered out"
        );
    }

    /// `file_version_value` cannot be reached without a real executable. Windows
    /// always ships `explorer.exe`, whose version resource carries both keys the
    /// friendly-name fallback consults.
    #[test]
    fn friendly_process_name_prefers_the_version_resource_over_the_stem() {
        let system_root = std::env::var("SystemRoot").expect("SystemRoot must be set on Windows");
        let explorer = format!("{system_root}\\explorer.exe");

        let name = friendly_process_name(&explorer, "explorer").expect("a display name");

        // The prettified stem would be exactly "Explorer"; the version resource
        // yields something richer, e.g. "Windows Explorer".
        assert!(!name.is_empty());
        assert_ne!(
            name, "Explorer",
            "the version resource must win over the stem"
        );
    }

    #[test]
    fn friendly_process_name_falls_back_to_the_stem_for_a_missing_binary() {
        let name = friendly_process_name("Z:\\definitely\\not\\here.exe", "my-cool_app");

        assert_eq!(name, Some("My Cool App".to_string()));
    }
}
