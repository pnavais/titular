[details]
name    = "Basic"
version = "1.0"
author  = "Pablo Navais"
url     = "https://github.com/pnavais/titular"

[vars]
f  = "*"
f2 = "*"
cl = "ca"
cr = "cl"

[pattern]
data = "${f:fg[cl]:pad}${m:fg[c2]:bg[cb2]}${f2:fg[cr]:pad}"
