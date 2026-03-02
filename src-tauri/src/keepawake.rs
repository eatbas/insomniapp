use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};

use crate::idle;
use crate::meeting;
use crate::state::AppState;

pub fn start_engine(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(3));
        let mut last_simulate = std::time::Instant::now();

        loop {
            check_interval.tick().await;

            let idle_secs = idle::get_idle_seconds();
            let in_meeting = meeting::is_in_meeting();

            let should_simulate = {
                let state = app.state::<AppState>();
                let mut status = state.status.lock().unwrap();
                status.idle_seconds = idle_secs;
                status.is_idle = idle_secs >= status.idle_threshold_secs;
                status.is_in_meeting = in_meeting;

                let should = status.enabled
                    && status.is_idle
                    && !status.is_in_meeting
                    && last_simulate.elapsed().as_secs() >= status.simulation_interval_secs;

                status.is_simulating = status.enabled && status.is_idle && !status.is_in_meeting;
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
