//! Idle, session-lock, and display-state queries, one implementation per
//! supported operating system.
//!
//! Each implementation exposes the same four functions: `get_idle_seconds`,
//! `is_session_locked`, `init_display_state_monitor`, and `is_display_on`.

mod convert;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::*;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod fallback;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub use self::fallback::*;
