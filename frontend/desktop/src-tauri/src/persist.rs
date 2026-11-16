//! Reading and writing the small JSON state files kept in the app's local data
//! directory.
//!
//! This module owns only the file I/O and its error text. Each persisting
//! module — the disguise name and the keep-awake settings — keeps its own
//! document handling in its covered store, and the directory itself comes from
//! `crate::paths`, which this module does not depend on.

use std::fs;
use std::path::Path;

/// Reads `file_name` from `dir`.
///
/// A missing or unreadable file yields `None`, so callers fall back to their
/// defaults instead of failing startup.
pub fn read_state_file(dir: &Path, file_name: &str) -> Option<String> {
    fs::read_to_string(dir.join(file_name)).ok()
}

/// Writes `contents` to `file_name` in `dir`, creating `dir` first.
///
/// `description` names the document in the error message, for example
/// `"disguise state"` gives `failed to write disguise state to <path>: <error>`.
pub fn write_state_file(
    dir: &Path,
    file_name: &str,
    contents: &str,
    description: &str,
) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create app data directory {}: {e}", dir.display()))?;

    let path = dir.join(file_name);

    fs::write(&path, contents)
        .map_err(|e| format!("failed to write {description} to {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const FILE_NAME: &str = "example_state.json";
    const DESCRIPTION: &str = "example state";

    #[test]
    fn a_written_file_reads_back() {
        let dir = tempdir().expect("temp dir");

        write_state_file(dir.path(), FILE_NAME, r#"{"key":1}"#, DESCRIPTION)
            .expect("write must succeed");

        assert_eq!(
            read_state_file(dir.path(), FILE_NAME),
            Some(r#"{"key":1}"#.to_string())
        );
    }

    #[test]
    fn reading_a_missing_file_yields_none() {
        let dir = tempdir().expect("temp dir");

        assert_eq!(read_state_file(dir.path(), FILE_NAME), None);
    }

    #[test]
    fn writing_creates_missing_parent_directories() {
        let dir = tempdir().expect("temp dir");
        let nested = dir.path().join("a").join("b");

        write_state_file(&nested, FILE_NAME, "{}", DESCRIPTION)
            .expect("write must create the directory");

        assert_eq!(read_state_file(&nested, FILE_NAME), Some("{}".to_string()));
    }

    #[test]
    fn writing_reports_the_description_and_path_when_the_target_is_not_writable() {
        let dir = tempdir().expect("temp dir");
        // A directory occupying the file's path makes `fs::write` fail.
        fs::create_dir(dir.path().join(FILE_NAME)).expect("create blocking directory");

        let error = write_state_file(dir.path(), FILE_NAME, "{}", DESCRIPTION)
            .expect_err("write must fail");

        assert!(error.contains("failed to write example state to"));
        assert!(error.contains(FILE_NAME));
    }

    #[test]
    fn writing_reports_the_directory_when_it_cannot_be_created() {
        let dir = tempdir().expect("temp dir");
        let blocking_file = dir.path().join("blocking");
        fs::write(&blocking_file, "not a directory").expect("write blocking file");

        // `create_dir_all` cannot create a directory beneath a regular file.
        let error = write_state_file(&blocking_file.join("nested"), FILE_NAME, "{}", DESCRIPTION)
            .expect_err("directory creation must fail");

        assert!(error.contains("failed to create app data directory"));
    }
}
