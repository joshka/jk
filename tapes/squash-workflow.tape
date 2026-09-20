Output target/dogfood-artifacts/betamax/squash-workflow.gif
Output target/dogfood-artifacts/betamax/squash-workflow-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1400
Set Height 900
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" squash"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Show

Caption "Mark the source, move to the destination, then choose Squash from Actions."
Type "\"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "Source change"
Sleep 700ms
Space
Sleep 500ms
Type "j"
Sleep 500ms
Type "a"
Wait+Screen "Squash"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-action-menu.png
State target/dogfood-artifacts/betamax/squash-workflow-action-menu.json
Type "s"
Wait+Screen "Source 1:"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-preview.png
State target/dogfood-artifacts/betamax/squash-workflow-preview.json

Caption "Esc cancels without losing the destination cursor or ordered source mark."
Escape
Wait+Screen "Destination change"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-cancelled.png
State target/dogfood-artifacts/betamax/squash-workflow-cancelled.json

Caption "Confirm the same exact whole-change command."
Type "a"
Type "s"
Wait+Screen "Source 1:"
Sleep 800ms
Enter
Wait+Screen "Squashed source changes"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-complete.png
State target/dogfood-artifacts/betamax/squash-workflow-complete.json

Caption "Command History records the squash and its resulting operation."
Type "C"
Wait+Screen "Command History"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-history.png
State target/dogfood-artifacts/betamax/squash-workflow-history.json

Caption "Open the captured operation for audit and recovery context."
Type "o"
Wait+Screen "Changed commits"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-operation.png
State target/dogfood-artifacts/betamax/squash-workflow-operation.json

Caption ""
Type "q"
Hide
Type "exit"
Enter
