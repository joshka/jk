Output target/dogfood-artifacts/betamax/action-menu-stability-20260918.gif

Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 800
Set Height 620
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set Framerate 15
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" stability"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Type "\"$JK_DEMO_BIN\" -R \"$JK_DEMO_REPO\""
Enter
Wait+Screen "jk jj"
Show

Caption "Describe keeps the terminal cursor beside the wrapped text."
Type "am"
Wait+Screen "Describe revision"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/action-menu-stability-before-20260918.png
State target/dogfood-artifacts/betamax/action-menu-stability-before-20260918.json
Type "!"
Sleep 700ms
Enter
Wait+Screen "Described revision"
Sleep 700ms

Caption "Reopening ignores jj's final newline; typing continues in the same place."
Type "am"
Wait+Screen "Describe revision"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/action-menu-stability-reopened-20260918.png
State target/dogfood-artifacts/betamax/action-menu-stability-reopened-20260918.json
Type " Again."
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/action-menu-stability-edited-20260918.png
State target/dogfood-artifacts/betamax/action-menu-stability-edited-20260918.json
Enter
Wait+Screen "Described revision"
Sleep 700ms

Caption "New selects the new change and shows a temporary completion toast."
Type "an"
Wait+Screen "Created new change"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/action-menu-stability-new-20260918.png
State target/dogfood-artifacts/betamax/action-menu-stability-new-20260918.json
Type "q"
Hide
Caption ""
Type "exit"
Enter
