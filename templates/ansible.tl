[details]
name    = "Ansible"
version = "1.1"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
fallback_palette = ""
f="*"
fe="$f"
surround_start="["
surround_end="]"
c="${theme_keyword:fallback_palette}"
c2="${theme_string:fallback_palette}"
c3="${theme_comment:fallback_palette}"

# Optional with display: `m | markup | color(...)` for rich inline/banner text.
[pattern]
data = "{{ m | color(name=c) | append(text=' ') }}{{ m2 | color(name=c2) | surround }}{{ fe | color(name=c3) | pad }}"
