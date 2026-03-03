#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
pub use self::windows::*;
#[cfg(target_os = "macos")]
pub use self::macos::*;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_session_locked() -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn init_display_state_monitor() {}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_display_on() -> bool {
    true
}
