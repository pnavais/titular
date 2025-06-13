[details]
name    = "Basic"
version = "1.0"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
main_color = "green"
message_color = "$main_color"
white="RGB(255,255,255)"
f="*"
fb="$f"
fe="${f2:f}"
c="$main_color"
c2="$message_color"
c3="$main_color"

[pattern]
data = "{{ fb | color(name=c) | pad }}{{ m | color(name=c2) }}{{ fe | color(name=c3) | pad }}"
