Output target/dogfood-artifacts/betamax/description-editor-20260919.gif

Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1200
Set Height 720
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set KeyboardOverlay Input
Set TypingSpeed 80ms
Set WaitTimeout 60s
Env NO_COLOR "1"

Hide
Type "cd /Users/joshka/local/jk/work/action-menu && JK_DEMO_REPO=$(bash scripts/action-menu-demo-fixture.sh) && export JK_DEMO_REPO"
Enter
Wait
Type "clear && exec target/debug/jk -R $JK_DEMO_REPO -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Sleep 1s
Show
Sleep 2s
Type "am"
Wait+Screen "Describe revision"
Sleep 3s
Home
Type "Safely "
Sleep 2s
End
Ctrl+j
Type "Keep cached forecasts fresh."
Sleep 2s
Home
Type "Goal: "
Sleep 3s
Screenshot target/dogfood-artifacts/betamax/description-editor-editing-20260919.png
Enter
Wait+Screen "Described revision"
Sleep 2s
Type "am"
Wait+Screen "Describe revision"
Sleep 4s
Screenshot target/dogfood-artifacts/betamax/description-editor-reopened-20260919.png
Escape
Sleep 2s
Hide
Type "q"
