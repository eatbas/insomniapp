use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::System::SystemInformation::GetTickCount;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

pub fn get_idle_seconds() -> u64 {
    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut lii).as_bool() {
            let now = GetTickCount();
            let idle_ms = now.wrapping_sub(lii.dwTime);
            (idle_ms / 1000) as u64
        } else {
            0
        }
    }
}

pub fn is_mic_active() -> bool {
    check_device_active("microphone")
}

pub fn is_camera_active() -> bool {
    check_device_active("webcam")
}

fn check_device_active(device: &str) -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let base_path = format!(
        "Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\{}",
        device
    );

    // Check NonPackaged apps (regular desktop apps)
    if let Ok(non_packaged) = hkcu.open_subkey(format!("{}\\NonPackaged", base_path)) {
        for key_name in non_packaged.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(app_key) = non_packaged.open_subkey(&key_name) {
                let stop_time: u64 = app_key.get_value("LastUsedTimeStop").unwrap_or(1);
                if stop_time == 0 {
                    return true;
                }
            }
        }
    }

    // Check packaged (UWP/Store) apps
    if let Ok(base) = hkcu.open_subkey(&base_path) {
        for key_name in base.enum_keys().filter_map(|k| k.ok()) {
            if key_name == "NonPackaged" {
                continue;
            }
            if let Ok(app_key) = base.open_subkey(&key_name) {
                let stop_time: u64 = app_key.get_value("LastUsedTimeStop").unwrap_or(1);
                if stop_time == 0 {
                    return true;
                }
            }
        }
    }

    false
}
