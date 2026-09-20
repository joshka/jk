Output target/dogfood-artifacts/betamax/design-proof-light.gif
Require jj
Set Shell "bash"
Set Theme "Builtin Light"
Set FontSize 20
Set Width 1400
Set Height 900
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
Type@0ms "printf '\\033]10;#000000\\007\\033]11;#ffffff\\007'"
Enter
Wait
Type@0ms "clear; \"$bin\" -R \"$fixture\" -n 12"
Enter
Wait+Screen "Show selected change details"
Show
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-light-graph.png
State target/dogfood-artifacts/betamax/design-proof-light-graph.json

Type "?"
Wait+Screen "Open and inspect:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-light-help.png
State target/dogfood-artifacts/betamax/design-proof-light-help.json

Escape
Type "d"
Wait+Screen "jk jj diff"
Wait+Screen "diff details"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-light-diff.png
State target/dogfood-artifacts/betamax/design-proof-light-diff.json

Type "H"
Wait+Screen "Show selected change details"
Space
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/design-proof-light-marked-graph.png
State target/dogfood-artifacts/betamax/design-proof-light-marked-graph.json
Type "j"
Type "a"
Wait+Screen "Squash"
Type "s"
Wait+Screen "Source 1:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-light-preview.png
State target/dogfood-artifacts/betamax/design-proof-light-preview.json

Escape
Wait+Screen "Show selected change details"
Hide
Type "q"
Wait
Type "exit"
Enter
