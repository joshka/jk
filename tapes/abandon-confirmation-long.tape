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
Type "cd /Users/joshka/local/jk/work/action-menu && JK_DEMO_REPO=$(bash scripts/abandon-dialog-fixture.sh 24) && export JK_DEMO_REPO"
Enter
Wait
Type "clear && exec target/debug/jk -R $JK_DEMO_REPO -n 8"
Enter
Wait+Screen "Tune forecast caching"
Sleep 1s
Type "jjaa"
Wait+Screen "Abandon this change?"
Sleep 1s
Show
Sleep 4s
Screenshot target/dogfood-artifacts/betamax/abandon-confirmation-long-top-20260919.png
PageDown
Sleep 2s
End
Sleep 3s
Screenshot target/dogfood-artifacts/betamax/abandon-confirmation-long-bottom-20260919.png
Type "n"
Sleep 1s
Hide
Type "q"
