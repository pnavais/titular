[details]
name    = "Box"
version = "1.0"
author  = "Pablo Navais"
url     = "https://github.com/pnavais/titular"

[vars]
f = "#"
f2 = " "
surround_start = "["
surround_end = "] "
c3 = "c2"
c4 = "c2"
c5 = "c2"
c6 = "c4"
c7 = "c3"
c8 = "c2"

[pattern]
data = """\
		${f:fg[c2]:pad}\n\
		${f:fg[c3]}${f2:pad:fg[c4]}${m:fg[c5]}${f2:pad:fg[c6]}${f:fg[c7]}\n\
		${f:fg[c8]:pad}\
	  """
