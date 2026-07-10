use std::fs;
use std::path::Path;

use super::name::{parse_persisted_name, serialize_persisted_name};

/// File name, inside the app's local data directory, holding the disguise.
const DISGUISE_STATE_FILE: &str = "disguise_state.json";

/// Loads the persisted disguise name from `dir`.
///
/// A missing, unreadable, or corrupt state file yields `None` so that startup
/// falls back to the undisguised default instead of failing.
pub fn load_name(dir: &Path) -> Option<String> {
    let content = fs::read_to_string(dir.join(DISGUISE_STATE_FILE)).ok()?;

    parse_persisted_name(&content)
}

/// Writes the disguise name to `dir`, creating the directory if needed.
///
/// Passing `None` persists the undisguised state.
pub fn save_name(dir: &Path, name: Option<&str>) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create app data directory {}: {e}", dir.display()))?;

    let path = dir.join(DISGUISE_STATE_FILE);

    fs::write(&path, serialize_persisted_name(name))
        .map_err(|e| format!("failed to write disguise state to {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn a_saved_name_loads_back() {
        let dir = tempdir().expect("temp dir");

        save_name(dir.path(), Some("Slack")).expect("save must succeed");

        assert_eq!(load_name(dir.path()), Some("Slack".to_string()));
    }

    #[test]
    fn saving_none_persists_the_undisguised_state() {
        let dir = tempdir().expect("temp dir");
        save_name(dir.path(), Some("Slack")).expect("save must succeed");

        save_name(dir.path(), None).expect("reset must succeed");

        assert_eq!(load_name(dir.path()), None);
        let raw = fs::read_to_string(dir.path().join(DISGUISE_STATE_FILE)).expect("file exists");
        assert_eq!(raw, r#"{"name":null}"#);
    }

    #[test]
    fn saving_creates_missing_parent_directories() {
        let dir = tempdir().expect("temp dir");
        let nested = dir.path().join("a").join("b");

        save_name(&nested, Some("Teams")).expect("save must create the directory");

        assert_eq!(load_name(&nested), Some("Teams".to_string()));
    }

    #[test]
    fn loading_from_a_directory_without_a_state_file_yields_none() {
        let dir = tempdir().expect("temp dir");

        assert_eq!(load_name(dir.path()), None);
    }

    #[test]
    fn loading_a_malformed_state_file_yields_none() {
        let dir = tempdir().expect("temp dir");
        fs::write(dir.path().join(DISGUISE_STATE_FILE), "{ not json").expect("write");

        assert_eq!(load_name(dir.path()), None);
    }

    #[test]
    fn saving_reports_the_path_when_the_target_is_not_writable() {
        let dir = tempdir().expect("temp dir");
        // A directory occupying the state file's path makes `fs::write` fail.
        fs::create_dir(dir.path().join(DISGUISE_STATE_FILE)).expect("create blocking directory");

        let error = save_name(dir.path(), Some("Slack")).expect_err("write must fail");

        assert!(error.contains("failed to write disguise state to"));
        assert!(error.contains(DISGUISE_STATE_FILE));
    }

    #[test]
    fn saving_reports_the_directory_when_it_cannot_be_created() {
        let dir = tempdir().expect("temp dir");
        let blocking_file = dir.path().join("blocking");
        fs::write(&blocking_file, "not a directory").expect("write blocking file");

        // `create_dir_all` cannot create a directory beneath a regular file.
        let error = save_name(&blocking_file.join("nested"), Some("Slack"))
            .expect_err("directory creation must fail");

        assert!(error.contains("failed to create app data directory"));
    }
}
