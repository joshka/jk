Output target/dogfood-artifacts/betamax/design-proof-standard.gif
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1080
Set Height 550
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Off
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env JK_DEMO_JJ_COLORS "1"

Hide
Type@0ms "unset NO_COLOR; source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" inspection"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "printf '\\033]10;#b4bcca\\007\\033]11;#0f141f\\007'"
Enter
Wait
Type@0ms "clear; \"$bin\" -R \"$fixture\" -n 12"
Enter
Sleep 500ms
State target/dogfood-artifacts/betamax/design-proof-standard-startup.json
Wait+Screen "Show selected change details"
Show
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-standard-graph.png
State target/dogfood-artifacts/betamax/design-proof-standard-graph.json

Type "?"
Wait+Screen "Open and inspect:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-standard-help.png
State target/dogfood-artifacts/betamax/design-proof-standard-help.json

Escape
Type "d"
Wait+Screen "jk jj diff"
Wait+Screen "diff details"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-standard-diff.png
State target/dogfood-artifacts/betamax/design-proof-standard-diff.json

Type "H"
Wait+Screen "Show selected change details"
Space
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/design-proof-standard-marked-graph.png
State target/dogfood-artifacts/betamax/design-proof-standard-marked-graph.json
Type "j"
Type "a"
Wait+Screen "Squash"
Type "s"
Wait+Screen "Source 1:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-standard-preview.png
State target/dogfood-artifacts/betamax/design-proof-standard-preview.json

Escape
Wait+Screen "Show selected change details"
Hide
Type "q"
Wait
Type "exit"
Enter
