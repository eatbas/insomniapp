extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(
        source_state_id: i32,
        event_type: u64,
    ) -> f64;
}

const CG_EVENT_SOURCE_STATE_COMBINED_SESSION: i32 = 0;
const CG_ANY_INPUT_EVENT_TYPE: u64 = u64::MAX;

pub fn get_idle_seconds() -> u64 {
    unsafe {
        CGEventSourceSecondsSinceLastEventType(
            CG_EVENT_SOURCE_STATE_COMBINED_SESSION,
            CG_ANY_INPUT_EVENT_TYPE,
        ) as u64
    }
}

pub fn is_session_locked() -> bool {
    false
}

pub fn init_display_state_monitor() {}

pub fn is_display_on() -> bool {
    true
}
