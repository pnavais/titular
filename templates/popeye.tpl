[details]
name    = "Popeye"
version = "1.0"
author  = "Pablo Navais"
url     = "https://github.com/pnavais/titular"

[vars]
f = "·"
c2 = "RGB(95,175,255)"
c3 = "c2"
c4 = "c2"
surround_start = "["
surround_end = "] "
error = "💥"
ok    =	"✅"
info  = "🔊"
warn  = "😱"
m3    = "0"
m4    = "0"
m5    = "0"
m6    = "0"
m7    = "100"

[pattern]
data = """\
		${m:fg[c2]+[ ]}%{m2:fg[c3]}${space:pad}${error} ${m3:fg[m3c]} ${warn} ${m4:fg[m4c]} ${info} ${m5:fg[m5c]} ${ok} ${m6:fg[m6c]} ${m7:fg[m7c]}%\n\
		${f:fg[c4]:pad}\n\
	   """
