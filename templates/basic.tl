[details]
name    = "Basic"
version = "1.1"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
# Uncolored when no theme is set (`-T` / `[templates].theme` empty); `theme_*` fills when set (`display` feature).
fallback_accent = ""
fallback_msg = ""
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
