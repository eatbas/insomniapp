//! Keep-awake engine.
//!
//! All decision logic lives in [`engine`] as a pure function. This module is a
//! thin adapter that samples the operating system, drives the async tick loop,
//! simulates the keypress, and emits status to the frontend. It is excluded
//! from coverage because every line of it is an unavoidable side effect: see
//! the coverage policy in `README.md`.

pub mod engine;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};

use crate::platform;
use crate::state::AppState;
use engine::{EngineTick, IdleTracking, CHECK_INTERVAL_SECS};

pub fn start_engine(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut check_interval = interval(Duration::from_secs(CHECK_INTERVAL_SECS));
        let mut last_simulate = Instant::now();
        let mut real_idle_start: Option<Instant> = None;

        loop {
            check_interval.tick().await;

            let tick = EngineTick {
                os_idle_secs: platform::get_idle_seconds(),
                is_session_locked: platform::is_session_locked(),
                is_display_on: platform::is_display_on(),
                secs_since_last_simulate: last_simulate.elapsed().as_secs(),
                tracked_idle_secs: real_idle_start.map(|start| start.elapsed().as_secs()),
            };

            let decision = {
                let state = app.state::<AppState>();
                let mut status = state.status.lock().unwrap();
                engine::evaluate_tick(&tick, &mut status)
            };

            match decision.idle_tracking {
                IdleTracking::Keep => {}
                IdleTracking::Clear => real_idle_start = None,
                // `checked_sub` yields `None` shortly after boot, when the
                // backdated instant would precede the monotonic clock's origin.
                // Leaving tracking unset is safe: the next tick falls back to
                // the OS idle counter and re-backdates from there.
                IdleTracking::StartBackdated(secs) => {
                    real_idle_start = Instant::now().checked_sub(Duration::from_secs(secs));
                }
            }

            if decision.should_simulate {
                simulate_f15();
                last_simulate = Instant::now();
            }

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
