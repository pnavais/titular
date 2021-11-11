[details]
name    = "Underline"
version = "1.0"
author  = "Pablo Navais"
url     = "https://github.com/pnavais/titular"

[vars]
f = "-"
c3 = "c2"
c4 = "c2"

[pattern]
data = """\
		${m:fg[c2]}%{m2:fg[c3]-[ ]}\n\
		${f:fg[c4]:fit}\
	   """
