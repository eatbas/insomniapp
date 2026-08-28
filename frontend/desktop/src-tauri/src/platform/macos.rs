//! Core Graphics idle query.
//!
//! `CGEventSourceSecondsSinceLastEventType` is an `unsafe` FFI call into the
//! window server whose result depends on the host session, so this file is
//! excluded from the coverage gate; the conversion it relies on lives in
//! [`super::convert`], which is covered. The smoke tests below still call each
//! wrapper for real. See the coverage policy in `README.md`.

use enigo::{Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

use super::convert::idle_seconds_from_cg_seconds;

extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(source_state_id: i32, event_type: u64) -> f64;
}

const CG_EVENT_SOURCE_STATE_COMBINED_SESSION: i32 = 0;
const CG_ANY_INPUT_EVENT_TYPE: u64 = u64::MAX;

pub fn get_idle_seconds() -> u64 {
    let seconds = unsafe {
        CGEventSourceSecondsSinceLastEventType(
            CG_EVENT_SOURCE_STATE_COMBINED_SESSION,
            CG_ANY_INPUT_EVENT_TYPE,
        )
    };

    idle_seconds_from_cg_seconds(seconds)
}

pub fn is_session_locked() -> bool {
    false
}

pub fn init_display_state_monitor() {}

pub fn is_display_on() -> bool {
    true
}

/// Holds the system awake. Not yet implemented on macOS.
///
/// The Windows implementation resets the operating system's power idle timers
/// directly. The macOS equivalent is an `IOPMAssertionCreateWithName` assertion
/// from IOKit, which is not yet wired up, so a Mac still relies purely on the
/// simulated input in [`crate::keepawake`] to stay awake.
pub fn hold_awake() {}

/// Resets the input idle counter with a relative pointer move of zero pixels.
///
/// The quiet default, mirroring the Windows implementation: it delivers no key
/// event and leaves the cursor where it is.
pub fn nudge_pointer() {
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.move_mouse(0, 0, Coordinate::Rel);
    }
}

/// Resets the input idle counter with a synthetic `F15` keypress.
///
/// Retained only as a fallback for input stacks that discard zero-delta pointer
/// moves. `F15` is a real key that every focused application can observe.
pub fn nudge_f15() {
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::F15, Direction::Click);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A year of idle time means the conversion has broken.
    const IMPLAUSIBLE_IDLE_SECS: u64 = 60 * 60 * 24 * 365;

    #[test]
    fn get_idle_seconds_reports_a_plausible_duration() {
        assert!(get_idle_seconds() < IMPLAUSIBLE_IDLE_SECS);
    }

    #[test]
    fn session_lock_and_display_state_are_not_yet_detected_on_macos() {
        // Both are hard-coded until macOS lock/display detection lands; the
        // engine treats them as "never pause".
        init_display_state_monitor();

        assert!(!is_session_locked());
        assert!(is_display_on());
    }

    #[test]
    fn holding_the_power_state_is_not_yet_implemented_on_macos() {
        hold_awake();
    }

    #[test]
    fn nudging_the_pointer_does_not_panic() {
        // A zero-delta move is unobservable, so it is safe to fire for real.
        // `nudge_f15` is deliberately left untested: it injects a real key into
        // whatever session the test runs in. `Enigo::new` failing on a headless
        // runner is a legitimate outcome the wrapper already absorbs.
        nudge_pointer();
    }
}
