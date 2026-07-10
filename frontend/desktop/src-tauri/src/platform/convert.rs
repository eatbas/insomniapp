//! Pure conversions between raw operating-system values and the units the
//! keep-awake engine works in. Compiled only on the platform that needs them.

/// Converts a pair of `GetTickCount` millisecond stamps to whole idle seconds.
///
/// `GetTickCount` wraps every 49.7 days; the wrapping subtraction keeps the
/// elapsed time correct across that boundary.
#[cfg(target_os = "windows")]
pub fn idle_seconds_from_ticks(now_ms: u32, last_input_ms: u32) -> u64 {
    u64::from(now_ms.wrapping_sub(last_input_ms) / 1000)
}

/// Interprets a `MONITOR_DISPLAY_STATE` value: 0 = off, 1 = on, 2 = dimmed.
///
/// A dimmed display still counts as on.
#[cfg(target_os = "windows")]
pub fn display_on_from_state(display_state: u32) -> bool {
    display_state != 0
}

/// Truncates the Core Graphics idle interval to whole seconds.
///
/// Anything that is not a positive, finite interval reports zero idle time. The
/// guard cannot be left to the `as` cast: that saturates, so `f64::INFINITY`
/// would become `u64::MAX` and the engine would treat the session as idle
/// forever and simulate immediately.
#[cfg(target_os = "macos")]
pub fn idle_seconds_from_cg_seconds(seconds: f64) -> u64 {
    if !seconds.is_finite() || seconds <= 0.0 {
        0
    } else {
        seconds as u64
    }
}

#[cfg(all(test, target_os = "windows"))]
mod windows_tests {
    use super::*;

    #[test]
    fn idle_seconds_truncates_towards_zero() {
        assert_eq!(idle_seconds_from_ticks(1_999, 0), 1);
        assert_eq!(idle_seconds_from_ticks(2_000, 0), 2);
    }

    #[test]
    fn idle_seconds_is_zero_when_input_just_happened() {
        assert_eq!(idle_seconds_from_ticks(500, 500), 0);
    }

    #[test]
    fn idle_seconds_survives_the_tick_count_wrap() {
        // 3 seconds elapsed across the 32-bit boundary.
        assert_eq!(idle_seconds_from_ticks(2_000, u32::MAX - 999), 3);
    }

    #[test]
    fn a_display_state_of_zero_is_off() {
        assert!(!display_on_from_state(0));
    }

    #[test]
    fn an_on_or_dimmed_display_is_on() {
        assert!(display_on_from_state(1));
        assert!(display_on_from_state(2));
    }
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use super::*;

    #[test]
    fn idle_seconds_truncates_the_fractional_part() {
        assert_eq!(idle_seconds_from_cg_seconds(3.99), 3);
    }

    #[test]
    fn idle_seconds_is_zero_for_a_just_delivered_event() {
        assert_eq!(idle_seconds_from_cg_seconds(0.0), 0);
    }

    #[test]
    fn a_negative_interval_reports_no_idle_time() {
        assert_eq!(idle_seconds_from_cg_seconds(-1.0), 0);
    }

    #[test]
    fn a_non_finite_interval_reports_no_idle_time() {
        // `f64::INFINITY as u64` saturates to `u64::MAX`, which would make the
        // engine believe the session had been idle for 584 billion years.
        assert_eq!(idle_seconds_from_cg_seconds(f64::INFINITY), 0);
        assert_eq!(idle_seconds_from_cg_seconds(f64::NEG_INFINITY), 0);
        assert_eq!(idle_seconds_from_cg_seconds(f64::NAN), 0);
    }
}
