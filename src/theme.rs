use strsim::jaro_winkler;
use syntect::highlighting::{Theme, ThemeSet};

use crate::{error::Result, utils};
use nu_ansi_term::Color::{Green, Yellow};

/// Ranks installed theme names for a typo or partial query (used for CLI hints).
///
/// Uses a prefix/substring boost plus Jaro–Winkler similarity from [`strsim`].
#[must_use]
pub(crate) fn rank_theme_name_suggestions<'a>(
    query: &str,
    theme_keys: impl Iterator<Item = &'a str>,
    limit: usize,
) -> Vec<&'a str> {
    let query = query.trim();
    if query.is_empty() || limit == 0 {
        return Vec::new();
    }
    let q = query.to_ascii_lowercase();
    let mut scored: Vec<(f64, &'a str)> = Vec::new();
    for key in theme_keys {
        if key.eq_ignore_ascii_case(query) {
            continue;
        }
        let k = key.to_ascii_lowercase();
        let jw = jaro_winkler(&q, &k);
        let mut score = jw * 200.0;
        if k.starts_with(&q) {
            score += 800.0;
        } else if k.contains(&q) {
            score += 400.0;
        }
        scored.push((score, key));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let Some(best) = scored.first().map(|(s, _)| *s) else {
        return Vec::new();
    };

    // With prefix/substring overlap, keep suggestions in the same rough band; for fuzzy-only,
    // require a strong Jaro–Winkler match so random queries do not list unrelated themes.
    let threshold = if best >= 400.0 {
        (best * 0.45).max(50.0)
    } else {
        170.0
    };

    scored
        .into_iter()
        .filter(|(s, _)| *s >= threshold)
        .take(limit)
        .map(|(_, k)| k)
        .collect()
}

pub struct ThemeManager {
    pub theme_set: ThemeSet,
}

impl ThemeManager {
    /// Loads built-in themes from the binary assets produced by the build script.
    ///
    /// # Errors
    /// Currently always returns `Ok`; reserved for future fallible initialization.
    pub fn init() -> Result<Self> {
        Ok(Self {
            theme_set: Self::load_themes(),
        })
    }

    ///
    /// This function loads the themes from the build script and returns them as a `ThemeSet`.
    ///
    /// # Returns
    /// The loaded `ThemeSet`.
    ///
    /// # Panics
    /// Panics if the embedded theme blob is corrupt or incompatible with the current `bincode` schema.
    fn load_themes() -> ThemeSet {
        // Load the serialized theme set from the build script
        let theme_set_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/theme_set.bin"));
        bincode::serde::decode_from_slice(theme_set_bytes, bincode::config::standard())
            .expect("theme_set.bin from build script should decode")
            .0
    }

    /// Lists the themes currently available in the binary.
    ///
    /// This function lists the themes currently available in the binary.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    /// Currently always returns `Ok(())`; reserved for future fallible output paths.
    pub fn list_themes(&self) -> Result<()> {
        let themes: Vec<&str> = self
            .theme_set
            .themes
            .keys()
            .map(std::string::String::as_str)
            .collect();
        utils::print_tree_with_prefixes(
            &themes,
            "theme",
            "Available themes",
            "\u{e22b}",
            "\u{f08b5}",
            |s| Yellow.paint(s).to_string(),
            |s| Green.paint(s).to_string(),
        );
        Ok(())
    }

    /// Resolves a theme by exact map key first, then ASCII case-insensitive key match.
    #[must_use]
    pub fn resolve_theme(&self, theme_name: &str) -> Option<&Theme> {
        self.theme_set.themes.get(theme_name).or_else(|| {
            self.theme_set
                .themes
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(theme_name))
                .map(|(_, theme)| theme)
        })
    }

    /// Closest theme names for an invalid query (for warning hints).
    #[must_use]
    pub fn suggest_theme_names(&self, query: &str, limit: usize) -> Vec<&str> {
        rank_theme_name_suggestions(
            query,
            self.theme_set.themes.keys().map(String::as_str),
            limit,
        )
    }

    /// Gets a theme from the theme set.
    ///
    /// # Arguments
    /// * `theme_name` - The name of the theme to get.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Panics
    /// Panics if no theme matches `theme_name` exactly or case-insensitively.
    #[must_use]
    pub fn get_theme(&self, theme_name: &str) -> &Theme {
        self.resolve_theme(theme_name).unwrap_or_else(|| {
            panic!("unknown syntax-highlighting theme: {theme_name}");
        })
    }
}

#[cfg(all(test, feature = "display"))]
mod tests {
    use super::{rank_theme_name_suggestions, ThemeManager};

    #[test]
    fn resolve_theme_matches_case_insensitively() {
        let mgr = ThemeManager::init().unwrap();
        let lower = mgr.resolve_theme("dracula");
        let upper = mgr.resolve_theme("DRACULA");
        assert!(lower.is_some() && upper.is_some());
        assert!(std::ptr::eq(lower.unwrap(), upper.unwrap()));
    }

    #[test]
    fn suggest_onehalf_prefix_prefers_half_variants() {
        let keys = ["Dracula", "OneHalfDark", "OneHalfLight", "Solarized (dark)"];
        let got = rank_theme_name_suggestions("OneHalf", keys.into_iter(), 4);
        assert_eq!(got.len(), 2);
        assert!(got.contains(&"OneHalfDark"));
        assert!(got.contains(&"OneHalfLight"));
    }

    #[test]
    fn suggest_fuzzy_typo_needs_high_similarity() {
        let keys = ["Dracula", "Monokai", "zzz-unrelated-theme-name"];
        let got = rank_theme_name_suggestions("Draculla", keys.into_iter(), 3);
        assert_eq!(got, vec!["Dracula"]);
    }

    #[test]
    fn suggest_empty_query_returns_empty() {
        assert!(rank_theme_name_suggestions("", ["a"].into_iter(), 3).is_empty());
    }
}
