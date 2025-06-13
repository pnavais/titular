[details]
name    = "Ansible"
version = "1.0"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
f="*"
fe="$f"
surround_start="["
surround_end="]"

[pattern]
data = "{{ m | color(name=c) | append(text=' ') }}{{ m2 | color(name=c2) | surround }}{{ fe | color(name=c3) | pad }}"
