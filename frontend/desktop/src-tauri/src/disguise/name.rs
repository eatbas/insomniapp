use serde_json::json;

/// Upper bound on a disguise name, in characters rather than bytes so that
/// multi-byte names are never truncated mid-character.
const DISGUISE_NAME_MAX_LEN: usize = 80;

/// Trims a user-supplied disguise name and caps its length.
///
/// Returns `None` when the name carries no visible characters, which callers
/// treat as "no disguise".
pub fn sanitize_name(name: &str) -> Option<String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(DISGUISE_NAME_MAX_LEN).collect())
    }
}

/// Reads the persisted `{"name": ...}` document.
///
/// A malformed document, a missing or non-string `name`, and a blank name all
/// yield `None`, so a corrupt state file degrades to the undisguised default
/// rather than failing startup.
pub fn parse_persisted_name(content: &str) -> Option<String> {
    let document = serde_json::from_str::<serde_json::Value>(content).ok()?;

    sanitize_name(document.get("name")?.as_str()?)
}

/// Renders the persisted document. Serialising a `Value` cannot fail.
pub fn serialize_persisted_name(name: Option<&str>) -> String {
    json!({ "name": name }).to_string()
}

/// Derives the per-disguise AppUserModelID that Windows uses to group taskbar
/// buttons and label notifications.
///
/// Every run of non-alphanumeric characters collapses to a single dot, so
/// `"Visual Studio Code"` becomes `"com.insomniapp.visual.studio.code"`.
#[cfg(target_os = "windows")]
pub fn app_user_model_id(name: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dot = false;

    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_was_dot = false;
        } else if !last_was_dot {
            slug.push('.');
            last_was_dot = true;
        }
    }

    let slug = slug.trim_matches('.');
    let slug = if slug.is_empty() { "insomniapp" } else { slug };

    format!("com.insomniapp.{slug}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_name_keeps_a_plain_name() {
        assert_eq!(sanitize_name("Slack"), Some("Slack".to_string()));
    }

    #[test]
    fn sanitize_name_trims_surrounding_whitespace() {
        assert_eq!(sanitize_name("  Slack \t\n"), Some("Slack".to_string()));
    }

    #[test]
    fn sanitize_name_rejects_an_empty_name() {
        assert_eq!(sanitize_name(""), None);
    }

    #[test]
    fn sanitize_name_rejects_a_whitespace_only_name() {
        assert_eq!(sanitize_name("   \t \n "), None);
    }

    #[test]
    fn sanitize_name_caps_an_overlong_name() {
        let sanitized = sanitize_name(&"a".repeat(DISGUISE_NAME_MAX_LEN + 20))
            .expect("a long name is not empty");

        assert_eq!(sanitized.chars().count(), DISGUISE_NAME_MAX_LEN);
    }

    #[test]
    fn sanitize_name_truncates_multi_byte_names_on_character_boundaries() {
        let sanitized = sanitize_name(&"é".repeat(DISGUISE_NAME_MAX_LEN + 5)).expect("not empty");

        assert_eq!(sanitized.chars().count(), DISGUISE_NAME_MAX_LEN);
    }

    #[test]
    fn parse_persisted_name_reads_a_stored_name() {
        assert_eq!(
            parse_persisted_name(r#"{"name":"Slack"}"#),
            Some("Slack".to_string())
        );
    }

    #[test]
    fn parse_persisted_name_sanitises_the_stored_name() {
        assert_eq!(
            parse_persisted_name(r#"{"name":"  Slack  "}"#),
            Some("Slack".to_string())
        );
    }

    #[test]
    fn parse_persisted_name_treats_a_null_name_as_undisguised() {
        assert_eq!(parse_persisted_name(r#"{"name":null}"#), None);
    }

    #[test]
    fn parse_persisted_name_treats_a_blank_name_as_undisguised() {
        assert_eq!(parse_persisted_name(r#"{"name":"   "}"#), None);
    }

    #[test]
    fn parse_persisted_name_rejects_a_missing_name_key() {
        assert_eq!(parse_persisted_name(r#"{"other":1}"#), None);
    }

    #[test]
    fn parse_persisted_name_rejects_a_non_string_name() {
        assert_eq!(parse_persisted_name(r#"{"name":123}"#), None);
    }

    #[test]
    fn parse_persisted_name_rejects_malformed_json() {
        assert_eq!(parse_persisted_name("{ this is not json"), None);
    }

    #[test]
    fn serialize_persisted_name_writes_a_name() {
        assert_eq!(
            serialize_persisted_name(Some("Slack")),
            r#"{"name":"Slack"}"#
        );
    }

    #[test]
    fn serialize_persisted_name_writes_null_when_undisguised() {
        assert_eq!(serialize_persisted_name(None), r#"{"name":null}"#);
    }

    #[test]
    fn a_serialised_name_round_trips_through_the_parser() {
        let document = serialize_persisted_name(Some("Visual Studio Code"));

        assert_eq!(
            parse_persisted_name(&document),
            Some("Visual Studio Code".to_string())
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn app_user_model_id_lowercases_and_dot_separates_words() {
        assert_eq!(
            app_user_model_id("Visual Studio Code"),
            "com.insomniapp.visual.studio.code"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn app_user_model_id_collapses_runs_of_separators() {
        assert_eq!(app_user_model_id("A --  B"), "com.insomniapp.a.b");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn app_user_model_id_strips_leading_and_trailing_separators() {
        assert_eq!(app_user_model_id("  !Slack!  "), "com.insomniapp.slack");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn app_user_model_id_falls_back_when_no_alphanumerics_survive() {
        assert_eq!(app_user_model_id("!!! ---"), "com.insomniapp.insomniapp");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn app_user_model_id_keeps_digits() {
        assert_eq!(app_user_model_id("Teams2"), "com.insomniapp.teams2");
    }
}
