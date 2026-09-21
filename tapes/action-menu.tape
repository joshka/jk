Output target/dogfood-artifacts/betamax/action-menu.gif
Output target/dogfood-artifacts/betamax/action-menu-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1200
Set Height 720
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" action"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Show

Caption "The action menu groups shipped change and recovery actions."
Type "\"$bin\" -R \"$fixture\" -n 3"
Enter
Wait+Screen "jk jj"
Sleep 800ms
Type "a"
Wait+Screen "Actions"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/action-menu-open.png
State target/dogfood-artifacts/betamax/action-menu-open.json

Caption "New runs immediately, selects the new working copy, records the command, refreshes the log, and keeps the footer controls."
Type "n"
Wait+Screen "Created new change"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/action-menu-new-success.png
State target/dogfood-artifacts/betamax/action-menu-new-success.json

Caption "Abandon keeps its destructive preview for a non-empty revision."
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "a"
Wait+Screen "Actions"
Type "a"
Wait+Screen "Abandon this change?"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/action-menu-abandon-preview.png
State target/dogfood-artifacts/betamax/action-menu-abandon-preview.json
Escape
Sleep 300ms
Type "q"
Sleep 300ms

Caption "Describe uses the terminal cursor and saves on Enter without a second confirmation."
Type "\"$bin\" -R \"$fixture\" -n 3"
Enter
Wait+Screen "jk jj"
Sleep 500ms
Type "a"
Wait+Screen "Actions"
Type "m"
Wait+Screen "Describe revision"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/action-menu-describe.png
State target/dogfood-artifacts/betamax/action-menu-describe.json
Type "Updated through inline describe"
Enter
Wait+Screen "Described revision"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/action-menu-describe-success.png
State target/dogfood-artifacts/betamax/action-menu-describe-success.json

Caption "Undo and Redo run immediately from the recovery section and confirm completion in a toast."
Type "a"
Wait+Screen "Actions"
Type "u"
Wait+Screen "Undid operation"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/action-menu-undo-success.png
State target/dogfood-artifacts/betamax/action-menu-undo-success.json
Type "a"
Wait+Screen "Actions"
Type "U"
Wait+Screen "Redid operation"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/action-menu-redo-success.png
State target/dogfood-artifacts/betamax/action-menu-redo-success.json
Type "q"
Hide
Type "exit"
Enter
