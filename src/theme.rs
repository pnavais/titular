use strsim::jaro_winkler;
use syntect::highlighting::{Theme, ThemeSet};

use crate::context::Context;
use crate::{error::Result, utils};
use nu_ansi_term::Color::{Green, Yellow};
use std::io::{self, Write};

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

/// Whether `s` counts as an explicit theme name from CLI/config (filters blanks and legacy `"null"`).
#[must_use]
pub fn theme_token_is_set(s: &str) -> bool {
    !s.trim().is_empty() && s != "null"
}

/// Theme for **rendered titles** (`theme_*` palette): CLI `-T` (`theme`), then `[templates].theme`.
#[must_use]
pub fn theme_name_for_template_palette(ctx: &Context) -> Option<&str> {
    ctx.get("theme")
        .filter(|s| theme_token_is_set(s))
        .or_else(|| ctx.get("templates.theme").filter(|s| theme_token_is_set(s)))
}

/// Theme for **fancy / preview highlighting**: CLI `-T` (`theme`), then `[defaults].display_theme`.
#[must_use]
pub fn theme_name_for_display_preview(ctx: &Context) -> Option<&str> {
    ctx.get("theme")
        .filter(|s| theme_token_is_set(s))
        .or_else(|| {
            ctx.get("defaults.display_theme")
                .filter(|s| theme_token_is_set(s))
        })
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
        let names = self.theme_names_sorted();
        let themes: Vec<&str> = names.iter().map(String::as_str).collect();
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

    /// Theme keys sorted case-insensitively (for `templates ls --themes` and `-o txt|json`).
    #[must_use]
    pub fn theme_names_sorted(&self) -> Vec<String> {
        let mut names: Vec<String> = self.theme_set.themes.keys().cloned().collect();
        names.sort_by_key(|a| a.to_ascii_lowercase());
        names
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

    /// Writes the same stderr warning and fuzzy “Did you mean?” hints used by fancy preview
    /// when `requested` is not a known theme and the caller falls back to `fallback_name`.
    pub fn warn_theme_not_found_using_fallback(&self, requested: &str, fallback_name: &str) {
        if requested == fallback_name {
            return;
        }
        let msg = format!(
            "WARN: syntax highlighting theme '{}' was not found; using default '{}'.",
            requested, fallback_name
        );
        let _ = writeln!(io::stderr(), "{}", Yellow.paint(msg));
        let hints = self.suggest_theme_names(requested, 3);
        if !hints.is_empty() {
            let hint_msg = format!("      Did you mean: {}?", hints.join(", "));
            let _ = writeln!(io::stderr(), "{}", Yellow.paint(hint_msg));
        }
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
    use super::{
        rank_theme_name_suggestions, theme_name_for_display_preview,
        theme_name_for_template_palette, ThemeManager,
    };
    use crate::context::Context;

    #[test]
    fn cli_theme_wins_for_both_resolution_helpers() {
        let mut ctx = Context::new();
        ctx.insert("templates.theme", "Dracula");
        ctx.insert("defaults.display_theme", "Catppuccin Mocha");
        ctx.insert("theme", "Monokai");
        assert_eq!(theme_name_for_template_palette(&ctx), Some("Monokai"));
        assert_eq!(theme_name_for_display_preview(&ctx), Some("Monokai"));
    }

    #[test]
    fn template_palette_uses_templates_theme_not_display_defaults() {
        let mut ctx = Context::new();
        ctx.insert("defaults.display_theme", "Dracula");
        assert_eq!(theme_name_for_template_palette(&ctx), None);

        ctx.insert("templates.theme", "Monokai");
        assert_eq!(theme_name_for_template_palette(&ctx), Some("Monokai"));
    }

    #[test]
    fn display_preview_uses_defaults_display_theme_not_templates_theme() {
        let mut ctx = Context::new();
        ctx.insert("templates.theme", "Monokai");
        assert_eq!(theme_name_for_display_preview(&ctx), None);

        ctx.insert("defaults.display_theme", "Dracula");
        assert_eq!(theme_name_for_display_preview(&ctx), Some("Dracula"));
    }

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
