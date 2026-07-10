//! Platform shims for targets without idle, lock, or display-state support.
//!
//! The app compiles and runs, but never reports the session as idle, so the
//! keep-awake engine stays dormant.

pub fn get_idle_seconds() -> u64 {
    0
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

    #[test]
    fn the_session_never_looks_idle() {
        assert_eq!(get_idle_seconds(), 0);
    }

    #[test]
    fn the_session_is_never_reported_as_locked() {
        assert!(!is_session_locked());
    }

    #[test]
    fn the_display_is_always_reported_as_on() {
        assert!(is_display_on());
    }

    #[test]
    fn initialising_the_display_monitor_is_a_no_op() {
        init_display_state_monitor();
    }
}
