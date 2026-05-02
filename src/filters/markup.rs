//! Rich terminal markup for title text (`display` feature).
//!
//! Grammar (processed after CLI `-e` escapes, if any):
//! - Line starts with `# ` (after leading whitespace): **h1** — bold + underline for the rest of the line.
//! - Line starts with `## ` : **h2** — bold only for the rest of the line.
//! - `**text**` — bold.
//! - `__text__` — underline.
//! - `//text//` — italic.
//! - `\` before `#`, `*`, `_`, or `/` removes special meaning for the following character.

use nu_ansi_term::Style;
use std::collections::HashMap;
use tera::{Error as TeraError, Value};

fn find_unescaped_pair(haystack: &str, a: u8, b: u8) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == a && bytes[i + 1] == b {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn apply_pair_delimited<F>(s: &str, open: u8, close: u8, style_fn: F) -> String
where
    F: Fn(&str) -> String,
{
    let mut out = String::with_capacity(s.len() + 16);
    let mut rest = s;
    while !rest.is_empty() {
        if let Some(pos) = find_unescaped_pair(rest, open, close) {
            out.push_str(&rest[..pos]);
            rest = &rest[pos + 2..];
            if let Some(end) = find_unescaped_pair(rest, open, close) {
                let inner = &rest[..end];
                out.push_str(&style_fn(inner));
                rest = &rest[end + 2..];
            } else {
                out.push(open as char);
                out.push(close as char);
                out.push_str(rest);
                break;
            }
        } else {
            out.push_str(rest);
            break;
        }
    }
    out
}

fn style_bold(chunk: &str) -> String {
    Style::new().bold().paint(chunk).to_string()
}

fn style_underline(chunk: &str) -> String {
    Style::new().underline().paint(chunk).to_string()
}

fn style_italic(chunk: &str) -> String {
    Style::new().italic().paint(chunk).to_string()
}

fn style_h1(chunk: &str) -> String {
    Style::new().bold().underline().paint(chunk).to_string()
}

fn style_h2(chunk: &str) -> String {
    Style::new().bold().paint(chunk).to_string()
}

fn apply_inline_markup(line: &str) -> String {
    let s = apply_pair_delimited(line, b'*', b'*', style_bold);
    let s = apply_pair_delimited(&s, b'_', b'_', style_underline);
    apply_pair_delimited(&s, b'/', b'/', style_italic)
}

/// Applies heading prefixes and inline markup line-by-line (preserves `\n`).
pub fn apply_rich_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 32);
    for line in input.split_inclusive('\n') {
        let (body, nl) = line.strip_suffix('\n').map_or((line, ""), |b| (b, "\n"));

        let trimmed = body.trim_start();
        let prefix_len = body.len() - trimmed.len();

        let styled_body = if let Some(rest) = trimmed.strip_prefix("## ") {
            let prefix = &body[..prefix_len];
            format!("{}{}", prefix, style_h2(&apply_inline_markup(rest)))
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            let prefix = &body[..prefix_len];
            format!("{}{}", prefix, style_h1(&apply_inline_markup(rest)))
        } else {
            apply_inline_markup(body)
        };

        out.push_str(&styled_body);
        out.push_str(nl);
    }
    out
}

/// Create the Tera `markup` filter.
pub fn create_markup_filter() -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError>
{
    move |value: &Value, _args: &HashMap<String, Value>| {
        let text = tera::try_get_value!("markup", "value", String, value);
        Ok(Value::String(apply_rich_markup(&text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bold_segment() {
        let s = apply_rich_markup("aa **bb** cc");
        assert!(s.contains("bb"));
        assert!(!s.contains("**"));
    }

    #[test]
    fn h2_prefix() {
        let s = apply_rich_markup("## Hello");
        assert!(!s.contains("## "));
        assert!(s.contains("Hello"));
    }

    #[test]
    fn escaped_stars() {
        let s = apply_rich_markup(r"a \**b**");
        assert!(s.contains("**b**"));
    }

    #[test]
    fn newline_preserved() {
        let s = apply_rich_markup("a\nb");
        assert_eq!(s.matches('\n').count(), 1);
    }
}
