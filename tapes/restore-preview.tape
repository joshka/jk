Output target/dogfood-artifacts/betamax/restore-preview.gif
Output target/dogfood-artifacts/betamax/restore-preview-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" workflows"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Show

Caption "Choose one source revision, then open Restore from the action menu."
Type "\"$bin\" -R \"$fixture_root/restore\" -n 3"
Enter
Wait+Screen "Restore working content"
Type "j"
Sleep 300ms
Type "a"
Wait+Screen "Actions"
Type "r"
Wait+Screen "Affected content: all paths"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-00-confirm.png
State target/dogfood-artifacts/betamax/restore-preview-00-confirm.json

Caption "Escape cancels without changing the repository or losing log context."
Escape
Wait+Screen "Restore base"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-01-cancelled.png
State target/dogfood-artifacts/betamax/restore-preview-01-cancelled.json

Caption "Confirming runs the displayed argv, refreshes the log, and exposes recovery."
Type "a"
Wait+Screen "Actions"
Type "r"
Wait+Screen "Affected content: all paths"
Enter
Wait+Screen "Restored all paths into working copy"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-02-success.png
State target/dogfood-artifacts/betamax/restore-preview-02-success.json

Caption "Command History retains the exact restore and resulting operation."
Type "C"
Wait+Screen "Command History"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-03-history.png
State target/dogfood-artifacts/betamax/restore-preview-03-history.json
Type "q"
Hide
Type "exit"
Enter
