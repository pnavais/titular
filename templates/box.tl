[details]
name    = "Box"
version = "1.1"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
fallback_accent = "green"
fallback_border = "green"
fallback_msg = "green"
accent_color = "${theme_keyword:fallback_accent}"
border_color = "${theme_comment:fallback_border}"
message_color = "${theme_foreground:fallback_msg}"
white="RGB(255,255,255)"
f="#"
f2="$space"
f3="$f"
f4="$f3"
fe="${f2:f}"
c="$border_color"
c2="$accent_color"
c3="$border_color"
c4="$message_color"
c5="$border_color"
c6="$accent_color"
c7="$border_color"

# Optional with display: `m | markup | color(name=c4)` for headings/markup in the box body.
[pattern]
data = """\
    {{ f | color(name=c) | pad }}\n\
    {{ f3 | color(name=c2) }}{{ f2 | color(name=c3) | pad }}{{ m | color(name=c4) }}{{ f2 | color(name=c5) | pad }}{{ f4 | color(name=c6)}}\n\
    {{ f | color(name=c7) | pad }}\n\
"""
