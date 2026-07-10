//! Core Graphics idle query.
//!
//! `CGEventSourceSecondsSinceLastEventType` is an `unsafe` FFI call into the
//! window server whose result depends on the host session, so this file is
//! excluded from the coverage gate; the conversion it relies on lives in
//! [`super::convert`], which is covered. The smoke tests below still call each
//! wrapper for real. See the coverage policy in `README.md`.

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
}
