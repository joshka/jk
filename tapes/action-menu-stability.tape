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
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type "cd /Users/joshka/local/jk/work/action-menu && export JK_DEMO_BIN=$PWD/target/debug/jk && export JK_DEMO_REPO=$(mktemp -d $PWD/target/action-menu-stability.XXXXXX) && jj git init $JK_DEMO_REPO"
Enter
Wait
Type "jj -R $JK_DEMO_REPO describe -m 'A description with enough words to wrap across the narrow editor and finish here'"
Enter
Wait
Type "$JK_DEMO_BIN -R $JK_DEMO_REPO"
Enter
Wait+Screen "jk jj"
Show

Caption "Describe keeps the terminal cursor beside the wrapped text."
Type "am"
Wait+Screen "Describe revision"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/action-menu-stability-before-20260918.png
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
Type " Again."
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/action-menu-stability-edited-20260918.png
Enter
Wait+Screen "Described revision"
Sleep 700ms

Caption "New selects the new change and shows a temporary completion toast."
Type "an"
Wait+Screen "Created new change"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/action-menu-stability-new-20260918.png
Type "q"
Hide
Caption ""
Type "exit"
Enter
