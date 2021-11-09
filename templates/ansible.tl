[details]
name    = "Ansible"
version = "1.0"
author  = "Pablo Navais"
url     = "https://github.com/pnavais/titular"

[vars]
f = "*"
surround_start = "["
surround_end = "] "

[pattern]
data = "${m:fg[c2]+[ ]}#{m2:fg[c3]}${f:fg[c4]:pad}"
