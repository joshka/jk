Output target/dogfood-artifacts/betamax/abandon-confirmation-long-20260919.gif

Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 920
Set Height 850
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set KeyboardOverlay Input
Set TypingSpeed 50ms
Set WaitTimeout 60s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" abandon 24"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Type "clear && exec \"$bin\" -R \"$JK_DEMO_REPO\" -n 8"
Enter
Wait+Screen "Tune forecast caching"
Sleep 1s
Type "jjaa"
Wait+Screen "Abandon this change?"
Sleep 1s
Show
Sleep 4s
Screenshot target/dogfood-artifacts/betamax/abandon-confirmation-long-top-20260919.png
State target/dogfood-artifacts/betamax/abandon-confirmation-long-top-20260919.json
PageDown
Sleep 2s
End
Sleep 3s
Screenshot target/dogfood-artifacts/betamax/abandon-confirmation-long-bottom-20260919.png
State target/dogfood-artifacts/betamax/abandon-confirmation-long-bottom-20260919.json
Type "n"
Sleep 1s
Hide
Type "q"
