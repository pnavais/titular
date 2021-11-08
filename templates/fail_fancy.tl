[details]
name    = "Fail fancy"
version = "1.0"
author  = "Pablo Navais"
url     = "https://github.com/pnavais/titular"

[vars]
f = "."
sign = "❌"

[pattern]
data = "${m:fg[c2]}${f:pad}${m2:fg[yellow]}${sign:fg[red]}"
