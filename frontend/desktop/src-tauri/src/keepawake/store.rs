//! Persistence for the keep-awake settings that must survive a restart.
//!
//! Only the nudge method is persisted. The two intervals are conveniences whose
//! defaults are harmless, but the `F15` fallback exists for sessions that
//! discard the pointer nudge, and silently reverting those users to the default
//! on every launch would leave them unprotected with no visible error.

use std::fs;
use std::path::Path;

use serde_json::json;

use crate::state::NudgeMethod;

/// File name, inside the app's local data directory, holding the settings.
const SETTINGS_FILE: &str = "keepawake_settings.json";

/// Reads the persisted `{"nudgeMethod": ...}` document.
///
/// A malformed document, a missing key, and an unrecognised method all yield
/// `None`, so a corrupt state file degrades to the default nudge rather than
/// failing startup.
pub fn parse_nudge_method(content: &str) -> Option<NudgeMethod> {
    let document = serde_json::from_str::<serde_json::Value>(content).ok()?;

    serde_json::from_value(document.get("nudgeMethod")?.clone()).ok()
}

/// Renders the persisted document. Serialising a `Value` cannot fail.
pub fn serialize_nudge_method(method: NudgeMethod) -> String {
    json!({ "nudgeMethod": method }).to_string()
}

/// Loads the persisted nudge method from `dir`.
pub fn load_nudge_method(dir: &Path) -> Option<NudgeMethod> {
    let content = fs::read_to_string(dir.join(SETTINGS_FILE)).ok()?;

    parse_nudge_method(&content)
}

/// Writes the nudge method to `dir`, creating the directory if needed.
pub fn save_nudge_method(dir: &Path, method: NudgeMethod) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create app data directory {}: {e}", dir.display()))?;

    let path = dir.join(SETTINGS_FILE);

    fs::write(&path, serialize_nudge_method(method))
        .map_err(|e| format!("failed to write keep-awake settings to {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn a_saved_method_loads_back() {
        let dir = tempdir().expect("temp dir");

        save_nudge_method(dir.path(), NudgeMethod::F15).expect("save must succeed");

        assert_eq!(load_nudge_method(dir.path()), Some(NudgeMethod::F15));
    }

    #[test]
    fn the_default_method_round_trips_too() {
        let dir = tempdir().expect("temp dir");

        save_nudge_method(dir.path(), NudgeMethod::MouseNudge).expect("save must succeed");

        assert_eq!(load_nudge_method(dir.path()), Some(NudgeMethod::MouseNudge));
    }

    #[test]
    fn the_persisted_document_uses_the_camel_case_wire_format() {
        assert_eq!(
            serialize_nudge_method(NudgeMethod::F15),
            r#"{"nudgeMethod":"f15"}"#
        );
    }

    #[test]
    fn saving_creates_missing_parent_directories() {
        let dir = tempdir().expect("temp dir");
        let nested = dir.path().join("a").join("b");

        save_nudge_method(&nested, NudgeMethod::F15).expect("save must create the directory");

        assert_eq!(load_nudge_method(&nested), Some(NudgeMethod::F15));
    }

    #[test]
    fn loading_from_a_directory_without_a_settings_file_yields_none() {
        let dir = tempdir().expect("temp dir");

        assert_eq!(load_nudge_method(dir.path()), None);
    }

    #[test]
    fn a_malformed_document_yields_none() {
        assert_eq!(parse_nudge_method("{ not json"), None);
    }

    #[test]
    fn a_document_without_the_key_yields_none() {
        assert_eq!(parse_nudge_method(r#"{"other":1}"#), None);
    }

    #[test]
    fn an_unrecognised_method_yields_none() {
        // A settings file written by a future version must not crash this one.
        assert_eq!(parse_nudge_method(r#"{"nudgeMethod":"f13"}"#), None);
    }

    #[test]
    fn saving_reports_the_path_when_the_target_is_not_writable() {
        let dir = tempdir().expect("temp dir");
        // A directory occupying the settings file's path makes `fs::write` fail.
        fs::create_dir(dir.path().join(SETTINGS_FILE)).expect("create blocking directory");

        let error = save_nudge_method(dir.path(), NudgeMethod::F15).expect_err("write must fail");

        assert!(error.contains("failed to write keep-awake settings to"));
        assert!(error.contains(SETTINGS_FILE));
    }

    #[test]
    fn saving_reports_the_directory_when_it_cannot_be_created() {
        let dir = tempdir().expect("temp dir");
        let blocking_file = dir.path().join("blocking");
        fs::write(&blocking_file, "not a directory").expect("write blocking file");

        // `create_dir_all` cannot create a directory beneath a regular file.
        let error = save_nudge_method(&blocking_file.join("nested"), NudgeMethod::F15)
            .expect_err("directory creation must fail");

        assert!(error.contains("failed to create app data directory"));
    }
}
