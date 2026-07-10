//! Pure helpers backing the Windows window/process enumeration in
//! [`super::enumerate`]. Kept free of Win32 calls so they can be tested.

/// Shell and system processes that own visible top-level windows but are never
/// something a user would recognise as a running application.
const NOISE_PROCESSES: [&str; 12] = [
    "ApplicationFrameHost",
    "ShellExperienceHost",
    "SearchHost",
    "StartMenuExperienceHost",
    "TextInputHost",
    "RuntimeBroker",
    "Widgets",
    "dwm",
    "sihost",
    "ctfmon",
    "taskhostw",
    "insomniapp",
];

/// Reports whether an executable stem belongs to the shell rather than to a
/// user-visible application.
pub fn is_noise_process(name: &str) -> bool {
    NOISE_PROCESSES
        .iter()
        .any(|blocked| blocked.eq_ignore_ascii_case(name))
}

/// Turns an executable stem into a display name when the binary carries no
/// version resource, e.g. `"my-cool_app"` becomes `"My Cool App"`.
///
/// Only the first character of each word is touched, so an internal capital in
/// `"myApp"` survives. Returns `None` when the stem holds nothing but
/// separators.
pub fn prettify_stem(stem: &str) -> Option<String> {
    let cleaned = stem.trim();

    if cleaned.is_empty() {
        return None;
    }

    // Runs of adjacent separators yield empty parts, which `filter_map` drops.
    let words: Vec<String> = cleaned
        .split(|c: char| c == '-' || c == '_' || c.is_whitespace())
        .filter_map(|part| {
            let mut characters = part.chars();
            let first = characters.next()?;
            Some(format!(
                "{}{}",
                first.to_ascii_uppercase(),
                characters.as_str()
            ))
        })
        .collect();

    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

/// Encodes a string as a NUL-terminated UTF-16 buffer for the `*W` Win32 APIs.
pub fn to_wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Interprets a `VarFileInfo\Translation` block as sorted, de-duplicated
/// (language, code page) pairs.
///
/// A trailing half-pair is discarded rather than being padded.
pub fn translation_pairs(words: &[u16]) -> Vec<(u16, u16)> {
    let mut pairs: Vec<(u16, u16)> = words
        .chunks_exact(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect();

    pairs.sort_unstable();
    pairs.dedup();
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_processes_are_noise() {
        assert!(is_noise_process("ApplicationFrameHost"));
        assert!(is_noise_process("dwm"));
        assert!(is_noise_process("insomniapp"));
    }

    #[test]
    fn noise_matching_ignores_case() {
        assert!(is_noise_process("SEARCHHOST"));
        assert!(is_noise_process("runtimebroker"));
    }

    #[test]
    fn real_applications_are_not_noise() {
        assert!(!is_noise_process("Code"));
        assert!(!is_noise_process(""));
    }

    #[test]
    fn prettify_stem_capitalises_a_single_word() {
        assert_eq!(prettify_stem("slack"), Some("Slack".to_string()));
    }

    #[test]
    fn prettify_stem_splits_on_hyphens_underscores_and_whitespace() {
        assert_eq!(
            prettify_stem("my-cool_app name"),
            Some("My Cool App Name".to_string())
        );
    }

    #[test]
    fn prettify_stem_preserves_inner_capitals() {
        assert_eq!(prettify_stem("myApp"), Some("MyApp".to_string()));
    }

    #[test]
    fn prettify_stem_collapses_repeated_separators() {
        assert_eq!(prettify_stem("a__b--c"), Some("A B C".to_string()));
    }

    #[test]
    fn prettify_stem_trims_surrounding_whitespace() {
        assert_eq!(prettify_stem("  slack  "), Some("Slack".to_string()));
    }

    #[test]
    fn prettify_stem_rejects_an_empty_stem() {
        assert_eq!(prettify_stem(""), None);
        assert_eq!(prettify_stem("   "), None);
    }

    #[test]
    fn prettify_stem_rejects_a_separator_only_stem() {
        assert_eq!(prettify_stem("-_-"), None);
    }

    #[test]
    fn prettify_stem_leaves_a_leading_digit_alone() {
        assert_eq!(prettify_stem("7zip"), Some("7zip".to_string()));
    }

    #[test]
    fn to_wide_null_appends_a_terminator() {
        assert_eq!(to_wide_null("AB"), vec![0x0041, 0x0042, 0x0000]);
    }

    #[test]
    fn to_wide_null_encodes_an_empty_string_as_just_the_terminator() {
        assert_eq!(to_wide_null(""), vec![0x0000]);
    }

    #[test]
    fn to_wide_null_encodes_astral_characters_as_surrogate_pairs() {
        assert_eq!(to_wide_null("\u{1F600}"), vec![0xD83D, 0xDE00, 0x0000]);
    }

    #[test]
    fn translation_pairs_reads_language_and_code_page_pairs() {
        assert_eq!(
            translation_pairs(&[0x0809, 0x04B0, 0x0409, 0x04B0]),
            vec![(0x0409, 0x04B0), (0x0809, 0x04B0)]
        );
    }

    #[test]
    fn translation_pairs_deduplicates() {
        assert_eq!(
            translation_pairs(&[0x0409, 0x04B0, 0x0409, 0x04B0]),
            vec![(0x0409, 0x04B0)]
        );
    }

    #[test]
    fn translation_pairs_discards_a_trailing_half_pair() {
        assert_eq!(
            translation_pairs(&[0x0409, 0x04B0, 0x0809]),
            vec![(0x0409, 0x04B0)]
        );
    }

    #[test]
    fn translation_pairs_of_an_empty_block_is_empty() {
        assert!(translation_pairs(&[]).is_empty());
    }
}
