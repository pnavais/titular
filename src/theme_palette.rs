//! Derive `RGB(...)` strings from a syntect [`Theme`] for template [`color`](crate::filters::color) vars.

use syntect::highlighting::{Color, Theme};

/// Semantic colors extracted from a TextMate-style theme (foreground only).
#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub foreground: String,
    pub comment: String,
    pub keyword: String,
    pub string: String,
    pub function: String,
    pub accent: String,
}

#[must_use]
pub fn rgb_string(c: Color) -> String {
    format!("RGB({},{},{})", c.r, c.g, c.b)
}

fn fallback_rgb(opt: Option<Color>, default: &str) -> String {
    opt.map(rgb_string).unwrap_or_else(|| default.to_string())
}

fn scope_blob(item: &syntect::highlighting::ThemeItem) -> String {
    let mut parts = Vec::new();
    for sel in &item.scope.selectors {
        for scope in sel.path.as_slice() {
            parts.push(scope.build_string());
        }
    }
    parts.join(" ")
}

fn scope_matches_atom(blob: &str, prefix: &str) -> bool {
    blob.split_whitespace()
        .any(|atom| atom == prefix || atom.starts_with(&format!("{prefix}.")))
}

fn pick(theme: &Theme, prefixes: &[&str], fallback: &str) -> String {
    for pfx in prefixes {
        for item in &theme.scopes {
            let blob = scope_blob(item);
            if scope_matches_atom(&blob, pfx) {
                if let Some(c) = item.style.foreground {
                    return rgb_string(c);
                }
            }
        }
    }
    fallback.to_string()
}

#[must_use]
pub fn palette_from_theme(theme: &Theme) -> ThemePalette {
    let fg = fallback_rgb(theme.settings.foreground, "RGB(200,200,200)");
    ThemePalette {
        foreground: fg.clone(),
        comment: pick(theme, &["comment", "comment.line"], &fg),
        keyword: pick(theme, &["keyword", "keyword.control", "storage.type"], &fg),
        string: pick(theme, &["string"], &fg),
        function: pick(
            theme,
            &[
                "entity.name.function",
                "support.function",
                "meta.function-call",
            ],
            &fg,
        ),
        accent: pick(
            theme,
            &["markup.heading", "entity.name.class", "variable.function"],
            &fg,
        ),
    }
}

impl ThemePalette {
    /// Inserts `theme_*` keys for use with `color(name=...)`.
    pub fn insert_into(&self, ctx: &mut crate::context::Context) {
        ctx.insert("theme_foreground", self.foreground.as_str());
        ctx.insert("theme_comment", self.comment.as_str());
        ctx.insert("theme_keyword", self.keyword.as_str());
        ctx.insert("theme_string", self.string.as_str());
        ctx.insert("theme_function", self.function.as_str());
        ctx.insert("theme_accent", self.accent.as_str());
    }
}
