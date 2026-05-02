//! Inject selected CLI flags from environment variables (bat-style), merged before real argv.
//! Flags already present in the **global** argv (tokens before `templates`) are not injected,
//! so explicit CLI wins without duplicate-flag errors.

use std::ffi::{OsStr, OsString};

/// True when `value` trims to `1`, `true`, or `yes` (ASCII case-insensitive).
pub(crate) fn truthy_env_var(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|s| truthy_env_var(&s))
}

fn push_flag_value(out: &mut Vec<OsString>, flag: &str, key: &str) {
    if let Ok(val) = std::env::var(key) {
        out.push(format!("{flag}={val}").into());
    }
}

fn global_argv_prefix(user_args: &[OsString]) -> &[OsString] {
    let pos = user_args
        .iter()
        .position(|a| a.as_os_str() == OsStr::new("templates"));
    match pos {
        Some(i) => &user_args[..i],
        None => user_args,
    }
}

#[derive(Clone, Copy, Default)]
struct GlobalClaims {
    template: bool,
    width: bool,
    #[cfg(feature = "display")]
    theme: bool,
    interpret_escapes: bool,
    no_newline: bool,
    with_time: bool,
    hide: bool,
    clear: bool,
}

fn scan_global_claims(global: &[OsString]) -> GlobalClaims {
    let mut claims = GlobalClaims::default();
    let mut i = 0;
    while i < global.len() {
        let cur = global[i].to_string_lossy();
        let s = cur.as_ref();

        if s == "--" {
            break;
        }

        // Long options
        if let Some(rest) = s.strip_prefix("--") {
            let (name, has_inline_value) = match rest.split_once('=') {
                Some((n, _)) => (n, true),
                None => (rest, false),
            };

            match name {
                "template" => claims.template = true,
                "width" => claims.width = true,
                #[cfg(feature = "display")]
                "theme" => claims.theme = true,
                "interpret-escapes" => claims.interpret_escapes = true,
                "no-newline" => claims.no_newline = true,
                "with-time" => claims.with_time = true,
                "hide" => claims.hide = true,
                "clear" => claims.clear = true,
                _ => {}
            }

            let takes_value = matches!(
                name,
                "template" | "width"
            ) || {
                #[cfg(feature = "display")]
                {
                    name == "theme"
                }
                #[cfg(not(feature = "display"))]
                {
                    false
                }
            };

            if takes_value && !has_inline_value {
                i += 1;
            }
            i += 1;
            continue;
        }

        // Short options (-w, -ew60, …)
        if let Some(rest) = s.strip_prefix('-') {
            if rest.is_empty() {
                i += 1;
                continue;
            }

            let alpha_prefix: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphabetic())
                .collect();

            // Exactly `-t`, `-w`, `-T`, `-e`, `-n`: next token may be the value (for twt/T).
            if alpha_prefix.len() == 1 && rest.len() == 1 {
                match alpha_prefix.chars().next().unwrap() {
                    't' => {
                        claims.template = true;
                        i += 2;
                        continue;
                    }
                    'w' => {
                        claims.width = true;
                        i += 2;
                        continue;
                    }
                    #[cfg(feature = "display")]
                    'T' => {
                        claims.theme = true;
                        i += 2;
                        continue;
                    }
                    'e' => {
                        claims.interpret_escapes = true;
                        i += 1;
                        continue;
                    }
                    'n' => {
                        claims.no_newline = true;
                        i += 1;
                        continue;
                    }
                    _ => {}
                }
            }

            for ch in alpha_prefix.chars() {
                match ch {
                    't' => claims.template = true,
                    'w' => claims.width = true,
                    #[cfg(feature = "display")]
                    'T' => claims.theme = true,
                    'e' => claims.interpret_escapes = true,
                    'n' => claims.no_newline = true,
                    _ => {}
                }
            }
            i += 1;
            continue;
        }

        i += 1;
    }

    claims
}

/// Synthetic argv fragments from env (excluding program name). Insert after `argv[0]`, before user args.
pub(crate) fn get_args_from_env_vars_filtered(user_args: &[OsString]) -> Vec<OsString> {
    let claims = scan_global_claims(global_argv_prefix(user_args));
    let mut args = Vec::new();

    if !claims.template {
        push_flag_value(&mut args, "--template", "TITULAR_TEMPLATE");
    }
    if !claims.width {
        push_flag_value(&mut args, "--width", "TITULAR_WIDTH");
    }

    #[cfg(feature = "display")]
    {
        if !claims.theme {
            if std::env::var_os("TITULAR_THEME").is_some() {
                push_flag_value(&mut args, "--theme", "TITULAR_THEME");
            } else {
                push_flag_value(&mut args, "--theme", "BAT_THEME");
            }
        }
    }

    if !claims.interpret_escapes && env_truthy("TITULAR_INTERPRET_ESCAPES") {
        args.push("--interpret-escapes".into());
    }
    if !claims.no_newline && env_truthy("TITULAR_NO_NEWLINE") {
        args.push("--no-newline".into());
    }
    if !claims.with_time && env_truthy("TITULAR_WITH_TIME") {
        args.push("--with-time".into());
    }
    if !claims.hide && env_truthy("TITULAR_HIDE") {
        args.push("--hide".into());
    }
    if !claims.clear && env_truthy("TITULAR_CLEAR") {
        args.push("--clear".into());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn truthy_env_var_cases() {
        assert!(truthy_env_var("1"));
        assert!(truthy_env_var(" true "));
        assert!(truthy_env_var("YES"));
        assert!(truthy_env_var("True"));
        assert!(!truthy_env_var(""));
        assert!(!truthy_env_var("0"));
        assert!(!truthy_env_var("no"));
        assert!(!truthy_env_var("maybe"));
    }

    #[test]
    fn env_injects_template_and_width_when_absent() {
        temp_env::with_vars(
            vec![
                ("TITULAR_TEMPLATE", Some("basic")),
                ("TITULAR_WIDTH", Some("50")),
            ],
            || {
                let args = get_args_from_env_vars_filtered(&[]);
                assert!(args.contains(&OsString::from("--template=basic")));
                assert!(args.contains(&OsString::from("--width=50")));
            },
        );
    }

    #[test]
    fn width_env_skipped_when_user_passes_w() {
        temp_env::with_vars(vec![("TITULAR_WIDTH", Some("50"))], || {
            let user = vec![
                OsString::from("-t"),
                OsString::from("ansible"),
                OsString::from("-e"),
                OsString::from("-m"),
                OsString::from("This is red"),
                OsString::from("-w"),
                OsString::from("60"),
            ];
            let env = get_args_from_env_vars_filtered(&user);
            assert!(
                !env.iter().any(|a| a.to_string_lossy().starts_with("--width")),
                "{env:?}"
            );
        });
    }

    #[cfg(feature = "display")]
    #[test]
    fn theme_env_skipped_when_user_passes_upper_t() {
        temp_env::with_vars(vec![("BAT_THEME", Some("ignored"))], || {
            let user = vec![
                OsString::from("-t"),
                OsString::from("ansible"),
                OsString::from("-T"),
                OsString::from("monokai"),
            ];
            let env = get_args_from_env_vars_filtered(&user);
            assert!(!env.iter().any(|a| a.to_string_lossy().starts_with("--theme")), "{env:?}");
        });
    }

    #[test]
    fn interpret_escapes_env_skipped_when_user_passes_e() {
        temp_env::with_vars(vec![("TITULAR_INTERPRET_ESCAPES", Some("true"))], || {
            let user = vec![OsString::from("-e"), OsString::from("-m"), OsString::from("x")];
            let env = get_args_from_env_vars_filtered(&user);
            assert!(!env.iter().any(|a| a == "--interpret-escapes"), "{env:?}");
        });
    }

    #[cfg(feature = "display")]
    #[test]
    fn titular_theme_overrides_bat_theme() {
        temp_env::with_vars(
            vec![
                ("TITULAR_THEME", Some("A")),
                ("BAT_THEME", Some("B")),
            ],
            || {
                let args = get_args_from_env_vars_filtered(&[]);
                let th: Vec<_> = args
                    .iter()
                    .filter_map(|a| a.to_str())
                    .filter(|s| s.starts_with("--theme="))
                    .collect();
                assert_eq!(th, vec!["--theme=A"]);
            },
        );
    }

    #[cfg(feature = "display")]
    #[test]
    fn bat_theme_used_when_titular_unset() {
        temp_env::with_vars(
            vec![
                ("TITULAR_THEME", None::<&str>),
                ("BAT_THEME", Some("Solarized")),
            ],
            || {
                let args = get_args_from_env_vars_filtered(&[]);
                assert!(
                    args.iter().any(|a| a.to_string_lossy() == "--theme=Solarized"),
                    "{args:?}"
                );
            },
        );
    }

    #[test]
    fn width_still_injected_for_templates_subcommand_when_not_in_global_prefix() {
        temp_env::with_vars(vec![("TITULAR_WIDTH", Some("50"))], || {
            let user = vec![
                OsString::from("templates"),
                OsString::from("list"),
                OsString::from("-o"),
                OsString::from("txt"),
            ];
            let env = get_args_from_env_vars_filtered(&user);
            assert!(env.contains(&OsString::from("--width=50")), "{env:?}");
        });
    }
}
