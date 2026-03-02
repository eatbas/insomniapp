use crate::platform;

pub fn get_idle_seconds() -> u64 {
    platform::get_idle_seconds()
}
