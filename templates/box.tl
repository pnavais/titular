[details]
name    = "Box"
version = "1.0"
author  = "pnavais"
url     = "https://github.com/pnavais"

[vars]
main_color = "green"
message_color = "$main_color"
white="RGB(255,255,255)"
f="#"
f2="$space"
f3="$f"
f4="$f3"
fe="${f2:f}"
c="$c"
c2="$c"
c3="$c"
c4="$c"
c5="$c3"
c6="$c2"
c7="$c"

[pattern]
data = """\
    {{ f | color(name=c) | pad }}\n\
    {{ f3 | color(name=c2) }}{{ f2 | color(name=c3) | pad }}{{ m | color(name=c4) }}{{ f2 | color(name=c5) | pad }}{{ f4 | color(name=c6)}}\n\
    {{ f | color(name=c7) | pad }}\n\
"""
