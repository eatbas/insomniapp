use serde::Serialize;
use std::sync::Mutex;

/// Smallest accepted value, in seconds, for either user-tunable interval.
///
/// A zero idle threshold would mark the session permanently idle, and a zero
/// simulation interval would fire a keypress on every engine tick. Values below
/// this minimum are rejected rather than clamped, so a malformed payload leaves
/// the previous setting intact.
pub const MIN_INTERVAL_SECS: u64 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub enabled: bool,
    pub is_idle: bool,
    pub idle_seconds: u64,
    pub is_session_locked: bool,
    pub is_display_off: bool,
    pub is_simulating: bool,
    pub idle_threshold_secs: u64,
    pub simulation_interval_secs: u64,
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            enabled: true,
            is_idle: false,
            idle_seconds: 0,
            is_session_locked: false,
            is_display_off: false,
            is_simulating: false,
            idle_threshold_secs: 30,
            simulation_interval_secs: 15,
        }
    }
}

impl AppStatus {
    /// Flips the keep-awake engine between enabled and disabled.
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Applies a partial settings update.
    ///
    /// Absent fields and values below [`MIN_INTERVAL_SECS`] are ignored, so a
    /// caller may update either interval independently of the other.
    pub fn apply_settings(
        &mut self,
        idle_threshold_secs: Option<u64>,
        simulation_interval_secs: Option<u64>,
    ) {
        if let Some(value) = idle_threshold_secs.filter(|secs| *secs >= MIN_INTERVAL_SECS) {
            self.idle_threshold_secs = value;
        }

        if let Some(value) = simulation_interval_secs.filter(|secs| *secs >= MIN_INTERVAL_SECS) {
            self.simulation_interval_secs = value;
        }
    }
}

pub struct AppState {
    pub status: Mutex<AppStatus>,
    pub disguise_name: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            status: Mutex::new(AppStatus::default()),
            disguise_name: Mutex::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_status_default_enables_the_engine_with_documented_intervals() {
        let status = AppStatus::default();

        assert!(status.enabled);
        assert!(!status.is_idle);
        assert_eq!(status.idle_seconds, 0);
        assert!(!status.is_session_locked);
        assert!(!status.is_display_off);
        assert!(!status.is_simulating);
        assert_eq!(status.idle_threshold_secs, 30);
        assert_eq!(status.simulation_interval_secs, 15);
    }

    #[test]
    fn toggle_enabled_switches_from_enabled_to_disabled() {
        let mut status = AppStatus::default();

        status.toggle_enabled();

        assert!(!status.enabled);
    }

    #[test]
    fn toggle_enabled_switches_from_disabled_to_enabled() {
        let mut status = AppStatus {
            enabled: false,
            ..AppStatus::default()
        };

        status.toggle_enabled();

        assert!(status.enabled);
    }

    #[test]
    fn apply_settings_updates_only_the_idle_threshold_when_given_one_field() {
        let mut status = AppStatus::default();

        status.apply_settings(Some(45), None);

        assert_eq!(status.idle_threshold_secs, 45);
        assert_eq!(status.simulation_interval_secs, 15);
    }

    #[test]
    fn apply_settings_updates_only_the_simulation_interval_when_given_one_field() {
        let mut status = AppStatus::default();

        status.apply_settings(None, Some(20));

        assert_eq!(status.idle_threshold_secs, 30);
        assert_eq!(status.simulation_interval_secs, 20);
    }

    #[test]
    fn apply_settings_updates_both_fields_together() {
        let mut status = AppStatus::default();

        status.apply_settings(Some(60), Some(25));

        assert_eq!(status.idle_threshold_secs, 60);
        assert_eq!(status.simulation_interval_secs, 25);
    }

    #[test]
    fn apply_settings_ignores_an_empty_payload() {
        let mut status = AppStatus::default();

        status.apply_settings(None, None);

        assert_eq!(status.idle_threshold_secs, 30);
        assert_eq!(status.simulation_interval_secs, 15);
    }

    #[test]
    fn apply_settings_accepts_the_minimum_interval() {
        let mut status = AppStatus::default();

        status.apply_settings(Some(MIN_INTERVAL_SECS), Some(MIN_INTERVAL_SECS));

        assert_eq!(status.idle_threshold_secs, MIN_INTERVAL_SECS);
        assert_eq!(status.simulation_interval_secs, MIN_INTERVAL_SECS);
    }

    #[test]
    fn apply_settings_rejects_values_below_the_minimum_interval() {
        let mut status = AppStatus::default();

        status.apply_settings(Some(0), Some(0));

        assert_eq!(status.idle_threshold_secs, 30);
        assert_eq!(status.simulation_interval_secs, 15);
    }

    #[test]
    fn app_status_is_debug_and_clone_for_the_status_update_payload() {
        let status = AppStatus::default();
        let cloned = status.clone();

        assert_eq!(cloned.idle_threshold_secs, status.idle_threshold_secs);
        assert!(format!("{status:?}").contains("idle_threshold_secs"));
    }

    #[test]
    fn app_status_serialises_to_the_camel_case_payload_the_frontend_expects() {
        let status = AppStatus::default();

        let json = serde_json::to_value(&status).expect("AppStatus must serialise");

        assert_eq!(json["enabled"], true);
        assert_eq!(json["isIdle"], false);
        assert_eq!(json["idleSeconds"], 0);
        assert_eq!(json["isSessionLocked"], false);
        assert_eq!(json["isDisplayOff"], false);
        assert_eq!(json["isSimulating"], false);
        assert_eq!(json["idleThresholdSecs"], 30);
        assert_eq!(json["simulationIntervalSecs"], 15);
    }

    #[test]
    fn app_state_default_starts_with_default_status_and_no_disguise() {
        let state = AppState::default();

        assert!(state.status.lock().unwrap().enabled);
        assert!(state.disguise_name.lock().unwrap().is_none());
    }
}
