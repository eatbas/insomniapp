use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Smallest accepted value, in seconds, for either user-tunable interval.
///
/// A zero idle threshold would mark the session permanently idle, and a zero
/// simulation interval would fire a nudge on every engine tick. Values below
/// this minimum are rejected rather than clamped, so a malformed payload leaves
/// the previous setting intact.
pub const MIN_INTERVAL_SECS: u64 = 1;

/// How the engine resets the operating system's input idle counter while the
/// session is idle.
///
/// Both methods reset `GetLastInputInfo`, which is what an inactivity-lock
/// policy and a chat client's presence indicator actually read. They differ
/// entirely in what else observes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NudgeMethod {
    /// A relative pointer move of zero pixels.
    ///
    /// The default. It produces no `WM_KEY*` message, matches no registered
    /// hotkey, reaches no low-level keyboard hook, and leaves the cursor where
    /// it is, so nothing in the session can react to it.
    MouseNudge,
    /// A synthetic `F15` keypress.
    ///
    /// The historical default, kept only as a fallback for remote-session and
    /// hypervisor input stacks that discard zero-delta pointer moves. `F15` is a
    /// real virtual key: every focused window, low-level keyboard hook, and
    /// registered hotkey sees it, which is why unhandled-key beeps and hotkey
    /// overlays fire once per simulation interval while it is selected.
    F15,
}

/// The wire strings [`NudgeMethod`] serialises to, in variant order.
///
/// `Serialize` is derived from `rename_all`, so these literals are duplicated
/// between the attribute and the hand-written `Deserialize` below.
/// `nudge_method_round_trips_through_the_camel_case_wire_format` asserts both
/// directions against the same strings, so the two cannot drift apart unnoticed.
const NUDGE_METHOD_VARIANTS: &[&str] = &["mouseNudge", "f15"];

/// Reads a [`NudgeMethod`] from its wire string.
///
/// This is written out rather than derived on purpose. A derived `Deserialize`
/// expands a generated identifier visitor whose `visit_u64` and `visit_bytes`
/// arms no JSON payload can reach, and `state.rs` is inside the 100% coverage
/// gate, so those arms would be uncoverable dead weight. Every branch below is
/// reachable from a JSON string and is covered by the tests in this module.
impl<'de> Deserialize<'de> for NudgeMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = String::deserialize(deserializer)?;

        match wire.as_str() {
            "mouseNudge" => Ok(Self::MouseNudge),
            "f15" => Ok(Self::F15),
            other => Err(de::Error::unknown_variant(other, NUDGE_METHOD_VARIANTS)),
        }
    }
}

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
    pub nudge_method: NudgeMethod,
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
            nudge_method: NudgeMethod::MouseNudge,
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
    /// Absent fields and interval values below [`MIN_INTERVAL_SECS`] are
    /// ignored, so a caller may update any one setting independently of the
    /// others. The nudge method has no invalid value to reject: the wire format
    /// admits only the two [`NudgeMethod`] variants, so an unknown string fails
    /// deserialisation before it reaches here.
    pub fn apply_settings(
        &mut self,
        idle_threshold_secs: Option<u64>,
        simulation_interval_secs: Option<u64>,
        nudge_method: Option<NudgeMethod>,
    ) {
        if let Some(value) = idle_threshold_secs.filter(|secs| *secs >= MIN_INTERVAL_SECS) {
            self.idle_threshold_secs = value;
        }

        if let Some(value) = simulation_interval_secs.filter(|secs| *secs >= MIN_INTERVAL_SECS) {
            self.simulation_interval_secs = value;
        }

        if let Some(value) = nudge_method {
            self.nudge_method = value;
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

    /// Asserts a method serialises to `wire` and reads back unchanged.
    fn assert_round_trip(method: NudgeMethod, wire: &str) {
        let json = serde_json::to_value(method).expect("NudgeMethod must serialise");
        assert_eq!(json, wire);

        let parsed: NudgeMethod =
            serde_json::from_value(json).expect("NudgeMethod must deserialise");
        assert_eq!(parsed, method);
    }

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

        status.apply_settings(Some(45), None, None);

        assert_eq!(status.idle_threshold_secs, 45);
        assert_eq!(status.simulation_interval_secs, 15);
    }

    #[test]
    fn apply_settings_updates_only_the_simulation_interval_when_given_one_field() {
        let mut status = AppStatus::default();

        status.apply_settings(None, Some(20), None);

        assert_eq!(status.idle_threshold_secs, 30);
        assert_eq!(status.simulation_interval_secs, 20);
    }

    #[test]
    fn apply_settings_updates_both_fields_together() {
        let mut status = AppStatus::default();

        status.apply_settings(Some(60), Some(25), None);

        assert_eq!(status.idle_threshold_secs, 60);
        assert_eq!(status.simulation_interval_secs, 25);
    }

    #[test]
    fn apply_settings_ignores_an_empty_payload() {
        let mut status = AppStatus::default();

        status.apply_settings(None, None, None);

        assert_eq!(status.idle_threshold_secs, 30);
        assert_eq!(status.simulation_interval_secs, 15);
    }

    #[test]
    fn apply_settings_accepts_the_minimum_interval() {
        let mut status = AppStatus::default();

        status.apply_settings(Some(MIN_INTERVAL_SECS), Some(MIN_INTERVAL_SECS), None);

        assert_eq!(status.idle_threshold_secs, MIN_INTERVAL_SECS);
        assert_eq!(status.simulation_interval_secs, MIN_INTERVAL_SECS);
    }

    #[test]
    fn apply_settings_rejects_values_below_the_minimum_interval() {
        let mut status = AppStatus::default();

        status.apply_settings(Some(0), Some(0), None);

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
        assert_eq!(json["nudgeMethod"], "mouseNudge");
    }

    #[test]
    fn the_default_nudge_method_is_the_silent_pointer_move() {
        // `F15` was the historical default. It reached every focused window,
        // low-level keyboard hook, and registered hotkey, which is the defect
        // this default exists to fix, so it must not drift back.
        assert_eq!(AppStatus::default().nudge_method, NudgeMethod::MouseNudge);
    }

    #[test]
    fn apply_settings_updates_only_the_nudge_method_when_given_one_field() {
        let mut status = AppStatus::default();

        status.apply_settings(None, None, Some(NudgeMethod::F15));

        assert_eq!(status.nudge_method, NudgeMethod::F15);
        assert_eq!(status.idle_threshold_secs, 30);
        assert_eq!(status.simulation_interval_secs, 15);
    }

    #[test]
    fn apply_settings_leaves_the_nudge_method_alone_when_it_is_absent() {
        let mut status = AppStatus {
            nudge_method: NudgeMethod::F15,
            ..AppStatus::default()
        };

        status.apply_settings(Some(45), Some(20), None);

        assert_eq!(status.nudge_method, NudgeMethod::F15);
    }

    #[test]
    fn nudge_method_round_trips_through_the_camel_case_wire_format() {
        // The frontend sends and receives these two strings verbatim, and the
        // derived `Serialize` and the hand-written `Deserialize` must agree on
        // them. Nothing else pins that agreement.
        assert_round_trip(NudgeMethod::MouseNudge, "mouseNudge");
        assert_round_trip(NudgeMethod::F15, "f15");

        // `Copy` is what lets the engine adapter read the method out of the
        // status lock without cloning the whole status.
        let copied = NudgeMethod::MouseNudge;
        assert_eq!(copied, NudgeMethod::MouseNudge);
        assert!(format!("{:?}", NudgeMethod::F15).contains("F15"));
    }

    #[test]
    fn an_unknown_nudge_method_is_rejected_with_the_permitted_variants() {
        let error = serde_json::from_str::<NudgeMethod>(r#""f13""#)
            .expect_err("an unknown variant must not deserialise");

        let message = error.to_string();
        for variant in NUDGE_METHOD_VARIANTS {
            assert!(message.contains(variant), "{message} must list {variant}");
        }
    }

    #[test]
    fn a_non_string_nudge_method_is_rejected() {
        // The wire format is a string; anything else fails in `String::deserialize`
        // before the variant match is reached.
        assert!(serde_json::from_str::<NudgeMethod>("15").is_err());
    }

    #[test]
    fn app_state_default_starts_with_default_status_and_no_disguise() {
        let state = AppState::default();

        assert!(state.status.lock().unwrap().enabled);
        assert!(state.disguise_name.lock().unwrap().is_none());
    }
}
