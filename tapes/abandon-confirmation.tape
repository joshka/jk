Output target/dogfood-artifacts/betamax/abandon-confirmation-v2-20260919.gif

Require cargo
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1400
Set Height 880
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 80ms
Set WaitTimeout 60s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" abandon"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Type "clear && exec \"$bin\" -R \"$JK_DEMO_REPO\" -n 8"
Enter
Wait+Screen "Tune forecast caching"
Sleep 1s
Type "jj"
Type "aa"
Wait+Screen "Abandon this change?"
Sleep 1s
Show
Sleep 5s
Screenshot target/dogfood-artifacts/betamax/abandon-confirmation-short-v2-20260919.png
State target/dogfood-artifacts/betamax/abandon-confirmation-short-v2-20260919.json
Left
Sleep 1s
Enter
Wait+Screen "Changes to abandon"
Sleep 4s
Down
Down
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/abandon-confirmation-diff-v2-20260919.png
State target/dogfood-artifacts/betamax/abandon-confirmation-diff-v2-20260919.json
Escape
Wait+Screen "Abandon this change?"
Sleep 2s
Tab
Sleep 1s
Right
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/abandon-confirmation-focused-v2-20260919.png
State target/dogfood-artifacts/betamax/abandon-confirmation-focused-v2-20260919.json
Space
Wait+Screen "Abandoned revision"
Sleep 3s
Hide
Type "q"
