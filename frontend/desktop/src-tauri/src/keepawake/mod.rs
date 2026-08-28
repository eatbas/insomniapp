//! Keep-awake engine.
//!
//! All decision logic lives in [`engine`] as a pure function. This module is a
//! thin adapter that samples the operating system, drives the async tick loop,
//! holds the power state, dispatches the configured nudge, and emits status to
//! the frontend. The nudges themselves live in [`crate::platform`] alongside the
//! other operating-system primitives, and settings persistence lives in
//! [`store`]; both are covered by tests. This module is excluded from coverage
//! because every line of it is an unavoidable side effect: see the coverage
//! policy in `README.md`.

pub mod engine;
mod store;

use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};

use crate::paths::app_data_dir;
use crate::platform;
use crate::state::{AppState, NudgeMethod};
use engine::{EngineTick, IdleTracking, CHECK_INTERVAL_SECS};

/// Restores the persisted nudge method into the shared status.
///
/// Must run before [`start_engine`]. A missing or corrupt settings file leaves
/// the default in place, so a bad file degrades to the quiet default rather
/// than failing startup.
pub fn restore_settings(app: &AppHandle) {
    let Ok(dir) = app_data_dir(app) else {
        return;
    };
    let Some(method) = store::load_nudge_method(&dir) else {
        return;
    };

    let state = app.state::<AppState>();
    let mut status = state.status.lock().unwrap();
    status.nudge_method = method;
}

/// Persists the nudge method so the `F15` fallback survives a restart.
pub fn persist_nudge_method(app: &AppHandle, method: NudgeMethod) -> Result<(), String> {
    store::save_nudge_method(&app_data_dir(app)?, method)
}

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

            // The nudge method is read under the same lock as the decision, so
            // a settings update cannot land between the two and have this tick
            // dispatch a nudge the decision was not made against.
            let (decision, nudge_method) = {
                let state = app.state::<AppState>();
                let mut status = state.status.lock().unwrap();
                let decision = engine::evaluate_tick(&tick, &mut status);
                (decision, status.nudge_method)
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

            if decision.should_hold_awake {
                platform::hold_awake();
            }

            if decision.should_simulate {
                match nudge_method {
                    NudgeMethod::MouseNudge => platform::nudge_pointer(),
                    NudgeMethod::F15 => platform::nudge_f15(),
                }
                last_simulate = Instant::now();
            }

            let status = app.state::<AppState>().status.lock().unwrap().clone();
            let _ = app.emit("status-update", &status);
        }
    });
}
