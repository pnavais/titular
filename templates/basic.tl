[details]
name    = "Basic"
version = "1.1"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
# Theme palette (`-T` / `defaults.display_theme`) when built with `display`; fallbacks keep plain builds usable.
fallback_accent = "green"
fallback_msg = "green"
main_color = "${theme_keyword:fallback_accent}"
message_color = "${theme_foreground:fallback_msg}"
white="RGB(255,255,255)"
f="*"
fb="$f"
fe="${f2:f}"
c="$main_color"
c2="$message_color"
c3="$main_color"

# Optional with display builds: pipe `m` through `markup` before `color` for **bold**, ## headings, etc.
[pattern]
data = "{{ fb | color(name=c) | pad }}{{ m | color(name=c2) }}{{ fe | color(name=c3) | pad }}"
