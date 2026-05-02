[details]
name    = "Fail"
version = "1.1"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
fail_color = "red"
debug_message_color = "yellow"
fallback_main = ""
f="."
c="${theme_foreground:fallback_main}"
c2="$debug_message_color"
c3="$debug_message_color"
c4="$fail_color"
sign="[FAIL]"
sign_fancy="❌"

# Optional with display: `markup` before `color` on `m` / `m2` for highlighted failure lines.
[pattern]
data = """{%- if fancy is defined and (fancy == "true" or fancy == "1") -%}\n\
               {% set fail_style = sign_fancy -%}\n\
          {%- else %}\n\
               {% set fail_style = sign -%}\n\
          {%- endif -%}\n\
          {{ m | color(name=c) }}{{ f | color(name=c2) | pad }}{{ m2 | color(name=c3) }}{{ fail_style | color(name=c4) | hide }}\
"""
