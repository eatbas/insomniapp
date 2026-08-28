//! Idle, session-lock, display-state, power-hold, and input-nudge primitives,
//! one implementation per supported operating system.
//!
//! Each implementation exposes the same seven functions, and nothing but this
//! comment binds them to that shape — a function added here and forgotten in one
//! of the targets fails only on that target's CI job, so this list must be kept
//! current alongside any signature change:
//!
//! - queries: `get_idle_seconds`, `is_session_locked`,
//!   `init_display_state_monitor`, `is_display_on`
//! - power hold: `hold_awake`
//! - input nudges: `nudge_pointer`, `nudge_f15`

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
