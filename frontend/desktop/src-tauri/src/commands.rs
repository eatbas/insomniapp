//! Tauri IPC surface.
//!
//! Every command here is a thin wrapper that unlocks shared state or forwards
//! to [`crate::disguise`]; the behaviour they expose is covered by the tests on
//! [`crate::state::AppStatus`] and the disguise helpers. The tests below pin the
//! one thing the wrappers own outright: the wire format of [`SettingsPayload`].
//!
//! Driving the commands through `tauri::test`'s `MockRuntime` was tried and
//! abandoned. Constructing any mock `App` links `muda`, which imports
//! `TaskDialogIndirect` from comctl32 **v6**. Only the bundled app receives an
//! activation-context manifest selecting v6, so the `cargo test` harness aborts
//! at load with `STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139) before a single test
//! runs. Cargo's `rustc-link-arg-tests` cannot patch this, as it applies to
//! `tests/` targets rather than to the lib's unit-test harness.
//!
//! This file therefore remains a documented coverage exclusion: see the coverage
//! policy in `README.md`.

use tauri::{AppHandle, State};

use crate::disguise::{self, DisguiseStatePayload};
use crate::keepawake;
use crate::state::{AppState, AppStatus, NudgeMethod};

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> AppStatus {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
pub fn toggle_enabled(state: State<'_, AppState>) -> AppStatus {
    let mut status = state.status.lock().unwrap();
    status.toggle_enabled();
    status.clone()
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPayload {
    pub idle_threshold_secs: Option<u64>,
    pub simulation_interval_secs: Option<u64>,
    pub nudge_method: Option<NudgeMethod>,
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: SettingsPayload,
) -> AppStatus {
    // The lock is released before touching the filesystem: a slow or failing
    // disk must not stall the engine tick that shares this mutex.
    let updated = {
        let mut status = state.status.lock().unwrap();
        status.apply_settings(
            settings.idle_threshold_secs,
            settings.simulation_interval_secs,
            settings.nudge_method,
        );
        status.clone()
    };

    // Only the nudge method is persisted, and only when this payload carried
    // one. A failed write leaves the running session correct, so it is not
    // worth failing the settings update over.
    if settings.nudge_method.is_some() {
        let _ = keepawake::persist_nudge_method(&app, updated.nudge_method);
    }

    updated
}

#[tauri::command]
pub fn open_disguise_window(app: AppHandle) -> Result<(), String> {
    disguise::open_disguise_window(&app)
}

#[tauri::command]
pub fn get_disguise_state(app: AppHandle) -> DisguiseStatePayload {
    disguise::get_state(&app)
}

#[tauri::command]
pub fn list_running_apps() -> Vec<String> {
    disguise::list_running_apps()
}

#[tauri::command]
pub fn apply_disguise(app: AppHandle, name: String) -> Result<(), String> {
    disguise::apply_disguise(&app, name)
}

#[tauri::command]
pub fn reset_disguise(app: AppHandle) -> Result<(), String> {
    disguise::reset_disguise(&app)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The frontend sends `invoke("update_settings", { settings: { ... } })`
    /// with camelCase keys. Nothing else pins that contract on the Rust side.
    fn parse(json: &str) -> SettingsPayload {
        serde_json::from_str(json).expect("the payload must deserialise")
    }

    #[test]
    fn settings_payload_reads_camel_case_keys() {
        let payload = parse(r#"{"idleThresholdSecs":45,"simulationIntervalSecs":20}"#);

        assert_eq!(payload.idle_threshold_secs, Some(45));
        assert_eq!(payload.simulation_interval_secs, Some(20));
        // An omitted `Option` field is absent rather than defaulted, so a
        // settings edit never silently rewrites the nudge method.
        assert_eq!(payload.nudge_method, None);
    }

    #[test]
    fn settings_payload_reads_both_nudge_methods() {
        assert_eq!(
            parse(r#"{"nudgeMethod":"mouseNudge"}"#).nudge_method,
            Some(NudgeMethod::MouseNudge)
        );
        assert_eq!(
            parse(r#"{"nudgeMethod":"f15"}"#).nudge_method,
            Some(NudgeMethod::F15)
        );
    }

    #[test]
    fn settings_payload_rejects_an_unknown_nudge_method() {
        // An unrecognised variant must fail deserialisation rather than quietly
        // selecting a default, so a typo in the frontend surfaces at once.
        let result = serde_json::from_str::<SettingsPayload>(r#"{"nudgeMethod":"f13"}"#);

        assert!(result.is_err());
    }

    /// `SettingsForm` debounces one edited field at a time, so each payload it
    /// sends carries exactly one key and omits the other entirely.
    #[test]
    fn settings_payload_allows_either_field_to_be_omitted() {
        let idle_only = parse(r#"{"idleThresholdSecs":45}"#);
        assert_eq!(idle_only.idle_threshold_secs, Some(45));
        assert_eq!(idle_only.simulation_interval_secs, None);

        let interval_only = parse(r#"{"simulationIntervalSecs":20}"#);
        assert_eq!(interval_only.idle_threshold_secs, None);
        assert_eq!(interval_only.simulation_interval_secs, Some(20));
    }

    #[test]
    fn settings_payload_treats_an_explicit_null_as_absent() {
        let payload = parse(r#"{"idleThresholdSecs":null,"simulationIntervalSecs":null}"#);

        assert_eq!(payload.idle_threshold_secs, None);
        assert_eq!(payload.simulation_interval_secs, None);
    }

    #[test]
    fn settings_payload_rejects_snake_case_keys() {
        // A regression here would silently ignore every settings update, so the
        // rename is asserted from both directions.
        let payload = parse(r#"{"idle_threshold_secs":45,"simulationIntervalSecs":20}"#);

        assert_eq!(payload.idle_threshold_secs, None);
        assert_eq!(payload.simulation_interval_secs, Some(20));
    }

    #[test]
    fn a_camel_case_payload_drives_the_documented_state_transition() {
        let mut status = AppStatus::default();
        let payload = parse(r#"{"idleThresholdSecs":45,"simulationIntervalSecs":0}"#);

        status.apply_settings(
            payload.idle_threshold_secs,
            payload.simulation_interval_secs,
            payload.nudge_method,
        );

        assert_eq!(status.idle_threshold_secs, 45);
        // The below-minimum interval is rejected, leaving the previous value.
        assert_eq!(status.simulation_interval_secs, 15);
    }
}
