Output target/dogfood-artifacts/betamax/design-proof-narrow.gif
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 820
Set Height 740
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
Wait+Screen "Show selected change details"
Show
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-narrow-graph.png
State target/dogfood-artifacts/betamax/design-proof-narrow-graph.json

Type "?"
Wait+Screen "Open and inspect:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-narrow-help.png
State target/dogfood-artifacts/betamax/design-proof-narrow-help.json

Escape
Type "d"
Wait+Screen "jk jj diff"
State target/dogfood-artifacts/betamax/design-proof-narrow-diff-ready.json
Wait+Screen "diff details"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-narrow-diff.png
State target/dogfood-artifacts/betamax/design-proof-narrow-diff.json

Type "H"
Wait+Screen "Show selected change details"
Space
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/design-proof-narrow-marked-graph.png
State target/dogfood-artifacts/betamax/design-proof-narrow-marked-graph.json
Type "j"
Type "a"
Wait+Screen "Squash"
Type "s"
Wait+Screen "Source 1:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-narrow-preview.png
State target/dogfood-artifacts/betamax/design-proof-narrow-preview.json

Escape
Wait+Screen "Show selected change details"
Hide
Type "q"
Wait
Type "exit"
Enter
