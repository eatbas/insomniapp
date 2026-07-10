//! Win32 idle, session-lock, and display-state queries.
//!
//! Every function here is an `unsafe` wrapper over a Win32 call whose result
//! depends on the host session, so no single run can exercise both branches of
//! [`is_session_locked`]. This file is therefore excluded from the coverage
//! gate; the arithmetic it relies on lives in [`super::convert`], which is
//! covered. The smoke tests below still call each wrapper for real. See the
//! coverage policy in `README.md`.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::Power::{
    PowerSettingRegisterNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, POWERBROADCAST_SETTING,
};
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, OpenInputDesktop, SwitchDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP,
};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::SystemServices::GUID_SESSION_DISPLAY_STATUS;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{DEVICE_NOTIFY_CALLBACK, PBT_POWERSETTINGCHANGE};

use super::convert::{display_on_from_state, idle_seconds_from_ticks};

static DISPLAY_MONITOR_INIT: Once = Once::new();
static DISPLAY_ON: AtomicBool = AtomicBool::new(true);

pub fn get_idle_seconds() -> u64 {
    unsafe {
        let mut last_input = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        if GetLastInputInfo(&mut last_input).as_bool() {
            idle_seconds_from_ticks(GetTickCount(), last_input.dwTime)
        } else {
            0
        }
    }
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

    let display_state = std::ptr::read_unaligned(power_setting.Data.as_ptr() as *const u32);
    DISPLAY_ON.store(display_on_from_state(display_state), Ordering::Relaxed);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A year of idle time means the tick arithmetic has broken.
    const IMPLAUSIBLE_IDLE_SECS: u64 = 60 * 60 * 24 * 365;

    #[test]
    fn get_idle_seconds_reports_a_plausible_duration() {
        assert!(get_idle_seconds() < IMPLAUSIBLE_IDLE_SECS);
    }

    #[test]
    fn is_session_locked_answers_without_panicking() {
        // Both outcomes are legitimate: an interactive desktop reports unlocked,
        // and a non-interactive CI session reports locked. Only the Win32 call
        // itself is under test.
        let _locked = is_session_locked();
    }

    #[test]
    fn the_display_monitor_is_idempotent() {
        // The `Once` must absorb the second call rather than registering, and
        // leaking, a second notification callback.
        init_display_state_monitor();
        init_display_state_monitor();

        // Windows hands the current display state to the callback the moment it
        // registers, so either value is legitimate on a headless CI host.
        let _on = is_display_on();
    }
}
