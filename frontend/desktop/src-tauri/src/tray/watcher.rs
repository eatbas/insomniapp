//! Keeps the tray icon registered with the Windows shell.
//!
//! The decision lives in [`super::registration`] as a pure function. This module
//! is the adapter that keeps time, asks the shell about the icon, and asks for
//! the icon again; none of it can run without a real windowing system, so it is
//! excluded from coverage: see the coverage policy in `README.md`.
//!
//! A Tauri tray handle is an `Rc` inside, which Tauri marks `Send` on the
//! promise that it is used on the main thread. The watcher task therefore only
//! keeps time: each look and each request is handed to the main thread through
//! [`on_main_thread`], and no tray handle is fetched, cloned, or dropped
//! anywhere else.

use std::time::Duration;

use tauri::AppHandle;
use tokio::sync::oneshot;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, RegisterWindowMessageW};

use super::registration::{Probe, RegistrationTracker};
use crate::disguise;

/// Wait before the first look, so a shell that is still starting has had time
/// to answer the registration made while the tray was set up.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(3);

/// Starts watching the tray icon's registration for the life of the process.
///
/// The loop ends when the tray is gone or the main thread takes no more work,
/// either of which means the application is shutting down.
pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut tracker = RegistrationTracker::default();
        let mut wait = FIRST_CHECK_DELAY;

        loop {
            tokio::time::sleep(wait).await;

            let Some(Some(probe)) = on_main_thread(&app, look_at).await else {
                return;
            };

            let decision = tracker.observe(probe);
            if decision.register_again {
                // Nothing useful can be done about a request that could not be
                // made: the next look finds the icon still missing and retries.
                let _ = on_main_thread(&app, request_registration).await;
            }
            wait = decision.next_check;
        }
    });
}

/// Runs `work` on the main thread and waits for its answer. `None` means the
/// main thread no longer takes work.
async fn on_main_thread<T, F>(app: &AppHandle, work: F) -> Option<T>
where
    T: Send + 'static,
    F: FnOnce(&AppHandle) -> T + Send + 'static,
{
    let (answer, result) = oneshot::channel();
    let app_on_main_thread = app.clone();
    app.run_on_main_thread(move || {
        // The watcher may have gone by the time this runs; nobody waits then.
        let _ = answer.send(work(&app_on_main_thread));
    })
    .ok()?;
    // A task dropped without running drops `answer`, which ends the wait.
    result.await.ok()
}

/// Asks the shell to set the tooltip the icon already has; the shell accepts an
/// update only for an icon it holds. `None` means there is no tray icon.
///
/// Main thread only, where Tauri runs the call inline: an error can only be the
/// shell's refusal.
fn look_at(app: &AppHandle) -> Option<Probe> {
    let tray = app.tray_by_id(disguise::TRAY_ID)?;
    let tooltip = disguise::current_app_name(app);

    Some(match tray.set_tooltip(Some(tooltip)) {
        Ok(()) => Probe::Registered,
        Err(_) => Probe::Missing,
    })
}

/// Asks the tray's own window to add the icon again, under the identity it
/// already has. Main thread only.
///
/// `tray-icon` adds its icon again when its window receives the shell's
/// `TaskbarCreated` message, so that message is posted to that one window;
/// nothing is broadcast. Building a second tray instead would give the icon a
/// new id, and Windows would forget where the user had placed it.
fn request_registration(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(disguise::TRAY_ID) else {
        return;
    };
    let Some(taskbar_created) = taskbar_created_message() else {
        return;
    };

    let _ = tray.with_inner_tray_icon(move |inner| {
        let tray_window = HWND(inner.window_handle());
        // SAFETY: the call takes plain values, and `tray_window` belongs to
        // `inner`, which is borrowed for the whole closure on the thread that
        // owns the window.
        let _ = unsafe { PostMessageW(Some(tray_window), taskbar_created, WPARAM(0), LPARAM(0)) };
    });
}

/// The message the shell broadcasts once its taskbar exists, or `None` when
/// Windows could not register it.
fn taskbar_created_message() -> Option<u32> {
    // SAFETY: `w!` yields a static, NUL-terminated wide string.
    let message = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
    (message != 0).then_some(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_taskbar_created_message_registers() {
        // Windows hands out registered window messages from 0xC000 to 0xFFFF.
        // Only this Win32 call is under test: everything else here needs a
        // live tray, which the test harness cannot construct.
        let message = taskbar_created_message().expect("TaskbarCreated should register");

        assert!((0xC000..=0xFFFF).contains(&message));
    }
}
