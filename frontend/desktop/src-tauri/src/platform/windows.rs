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
    PowerSettingRegisterNotification, SetThreadExecutionState, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS,
    ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, POWERBROADCAST_SETTING,
};
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, OpenInputDesktop, SwitchDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP,
};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::SystemServices::GUID_SESSION_DISPLAY_STATUS;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetLastInputInfo, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, LASTINPUTINFO, MOUSEEVENTF_MOVE, MOUSEINPUT, VK_F15,
};
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

/// Resets the system and display power idle timers for the current tick.
///
/// The flags are passed deliberately **without** `ES_CONTINUOUS`. The continuous
/// form attaches persistent state to the calling thread, and the engine runs
/// inside a `tauri::async_runtime` task whose worker thread may differ between
/// ticks, so a continuous assertion could be stranded on a thread that later
/// parks or exits. Passing the flags alone resets both idle timers once and
/// holds nothing, which is correct to repeat every tick from whichever thread
/// happens to run it.
///
/// A side effect of holding nothing is that the app never appears in
/// `powercfg /requests`; an empty listing there is the expected result rather
/// than evidence the hold has failed.
///
/// The call returns the previous `EXECUTION_STATE`, or zero on failure. Nothing
/// useful can be done about a failure to reset the timers, so the result is
/// discarded and the next tick simply tries again.
pub fn hold_awake() {
    unsafe {
        let _ = SetThreadExecutionState(ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
    }
}

/// Resets the input idle counter with a relative pointer move of zero pixels.
///
/// This is the quietest event that still resets `GetLastInputInfo`, which is
/// what an inactivity-lock policy and a chat client's presence indicator read.
/// It produces no `WM_KEY*` message, so no focused window sees a keystroke; it
/// matches no `RegisterHotKey` binding and reaches no `WH_KEYBOARD_LL` hook; and
/// the zero delta leaves the cursor where it is and gives raw-input consumers
/// such as games nothing to act on.
///
/// `SendInput` returns the number of events queued, or zero on failure. Nothing
/// useful can be done here about a rejected event, and the next tick simply
/// tries again, so the count is discarded.
pub fn nudge_pointer() {
    let movement = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                ..Default::default()
            },
        },
    };

    unsafe {
        let _ = SendInput(&[movement], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Resets the input idle counter with a synthetic `F15` keypress.
///
/// Retained only as a fallback for remote-session and hypervisor input stacks
/// that discard zero-delta pointer moves. Unlike [`nudge_pointer`], `F15` is a
/// real virtual key: the focused window, every low-level keyboard hook, and
/// every registered hotkey all see it, which is why selecting it can produce an
/// unhandled-key beep or a hotkey overlay once per simulation interval.
pub fn nudge_f15() {
    // Both events are queued in one call so no other application's input can be
    // interleaved between the press and the release.
    let events = [f15_event(KEYBD_EVENT_FLAGS(0)), f15_event(KEYEVENTF_KEYUP)];

    unsafe {
        let _ = SendInput(&events, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Builds one `F15` keyboard event: `KEYBD_EVENT_FLAGS(0)` presses the key and
/// `KEYEVENTF_KEYUP` releases it.
fn f15_event(flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VK_F15,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
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
    fn holding_the_power_state_does_not_panic() {
        // The one-shot form holds nothing a later call could observe, so only
        // the Win32 call itself is under test.
        hold_awake();
    }

    #[test]
    fn nudging_the_pointer_does_not_panic() {
        // A zero-delta move is unobservable: it changes no cursor position and
        // delivers no keystroke, so it is safe to fire for real in CI.
        //
        // `nudge_f15` is deliberately left untested. It is the same `SendInput`
        // call shape, but it injects a real virtual key into whatever session
        // the test happens to run in, which is not worth the marginal coverage
        // of an already-excluded file.
        nudge_pointer();
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
