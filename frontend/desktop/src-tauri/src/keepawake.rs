use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};

use crate::idle;
use crate::meeting;
use crate::platform;
use crate::state::AppState;

pub fn start_engine(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(3));
        let mut last_simulate = std::time::Instant::now();
        let mut real_idle_start: Option<std::time::Instant> = None;

        loop {
            check_interval.tick().await;

            let os_idle_secs = idle::get_idle_seconds();
            let in_meeting = meeting::is_in_meeting();
            let is_session_locked = platform::is_session_locked();
            let is_display_on = platform::is_display_on();

            // Detect genuine user input: OS idle is low AND we didn't just simulate.
            // Grace period (5s) > check interval (3s) to avoid false positives
            // from our own F15 simulation resetting the OS idle timer.
            let user_became_active = os_idle_secs < 5 && last_simulate.elapsed().as_secs() > 5;

            if user_became_active {
                real_idle_start = None;
            }

            // Use tracked idle time if we're in "known idle" state, else use OS idle
            let effective_idle_secs = match real_idle_start {
                Some(start) => start.elapsed().as_secs(),
                None => os_idle_secs,
            };

            let should_simulate = {
                let state = app.state::<AppState>();
                let mut status = state.status.lock().unwrap();
                status.idle_seconds = effective_idle_secs;
                status.is_idle = effective_idle_secs >= status.idle_threshold_secs;
                status.is_in_meeting = in_meeting;
                status.is_session_locked = is_session_locked;
                status.is_display_off = !is_display_on;

                // Record idle start when first crossing threshold (backdated)
                if status.is_idle && real_idle_start.is_none() {
                    real_idle_start = Some(
                        std::time::Instant::now() - Duration::from_secs(os_idle_secs),
                    );
                }

                // Reset tracking when app is disabled
                if !status.enabled {
                    real_idle_start = None;
                }

                let should = status.enabled
                    && status.is_idle
                    && !status.is_in_meeting
                    && !status.is_session_locked
                    && !status.is_display_off
                    && last_simulate.elapsed().as_secs() >= status.simulation_interval_secs;

                status.is_simulating = status.enabled
                    && status.is_idle
                    && !status.is_in_meeting
                    && !status.is_session_locked
                    && !status.is_display_off;
                should
            };

            if should_simulate {
                simulate_f15();
                last_simulate = std::time::Instant::now();
            }

            // Emit status to frontend
            let status = app.state::<AppState>().status.lock().unwrap().clone();
            let _ = app.emit("status-update", &status);
        }
    });
}

fn simulate_f15() {
    if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
        let _ = enigo.key(Key::F15, Direction::Click);
    }
}
