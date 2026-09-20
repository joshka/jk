Output target/dogfood-artifacts/betamax/design-proof-compact.gif
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 820
Set Height 390
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
Screenshot target/dogfood-artifacts/betamax/design-proof-compact-graph.png
State target/dogfood-artifacts/betamax/design-proof-compact-graph.json

Type "?"
Wait+Screen "Open and inspect:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-compact-help.png
State target/dogfood-artifacts/betamax/design-proof-compact-help.json

Escape
Wait+Screen "Show selected change details"
Space
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/design-proof-compact-marked-graph.png
State target/dogfood-artifacts/betamax/design-proof-compact-marked-graph.json
Type "j"
Type "a"
Wait+Screen "Actions"
Type "s"
Wait+Screen "Source 1:"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/design-proof-compact-preview.png
State target/dogfood-artifacts/betamax/design-proof-compact-preview.json
Wait+Screen "scroll"
PageDown
Wait+Screen "--use-destination-message"
Sleep 650ms
Screenshot target/dogfood-artifacts/betamax/design-proof-compact-preview-scrolled.png
State target/dogfood-artifacts/betamax/design-proof-compact-preview-scrolled.json

Escape
Wait+Screen "Show selected change details"
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/design-proof-compact-cancelled.png
State target/dogfood-artifacts/betamax/design-proof-compact-cancelled.json
Hide
Type "q"
Wait
Type "exit"
Enter
