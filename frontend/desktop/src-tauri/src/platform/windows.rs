use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::Power::{
    PowerSettingRegisterNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, POWERBROADCAST_SETTING,
};
use windows::Win32::System::SystemServices::GUID_SESSION_DISPLAY_STATUS;
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, OpenInputDesktop, SwitchDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::WindowsAndMessaging::{
    DEVICE_NOTIFY_CALLBACK, PBT_POWERSETTINGCHANGE,
};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

static DISPLAY_MONITOR_INIT: Once = Once::new();
static DISPLAY_ON: AtomicBool = AtomicBool::new(true);

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

pub fn init_display_state_monitor() {
    DISPLAY_MONITOR_INIT.call_once(|| unsafe {
        let params = Box::into_raw(Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(display_power_callback),
            Context: std::ptr::null_mut(),
        }));

        let recipient = HANDLE(params.cast::<c_void>());
        let mut registration_handle = std::ptr::null_mut();
        let result = PowerSettingRegisterNotification(
            &GUID_SESSION_DISPLAY_STATUS,
            DEVICE_NOTIFY_CALLBACK,
            recipient,
            &mut registration_handle,
        );

        if result != WIN32_ERROR(0) {
            let _ = Box::from_raw(params);
        }
    });
}

pub fn is_display_on() -> bool {
    DISPLAY_ON.load(Ordering::Relaxed)
}

pub fn is_session_locked() -> bool {
    unsafe {
        let Ok(input_desktop) =
            OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_SWITCHDESKTOP)
        else {
            return true;
        };

        let can_switch = SwitchDesktop(input_desktop).is_ok();
        let _ = CloseDesktop(input_desktop);
        !can_switch
    }
}

unsafe extern "system" fn display_power_callback(
    _context: *const c_void,
    power_broadcast_type: u32,
    setting: *const c_void,
) -> u32 {
    if power_broadcast_type != PBT_POWERSETTINGCHANGE {
        return 0;
    }

    if setting.is_null() {
        return 0;
    }

    let power_setting = &*(setting as *const POWERBROADCAST_SETTING);
    if power_setting.PowerSetting != GUID_SESSION_DISPLAY_STATUS {
        return 0;
    }

    if power_setting.DataLength < std::mem::size_of::<u32>() as u32 {
        return 0;
    }

    // MONITOR_DISPLAY_STATE: 0 = off, 1 = on, 2 = dimmed.
    let display_state = std::ptr::read_unaligned(power_setting.Data.as_ptr() as *const u32);
    DISPLAY_ON.store(display_state != 0, Ordering::Relaxed);
    0
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
