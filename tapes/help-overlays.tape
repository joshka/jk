Output target/dogfood-artifacts/betamax/help-overlays.gif
Output target/dogfood-artifacts/betamax/help-overlays-final-state.json

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
Set KeyboardOverlay Input
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" inspection"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Show

Caption "Press ? to open contextual help for the current screen."
Type "\"$bin\" -R \"$fixture\" -n 8"
Enter
Wait+Screen "jk jj"
Sleep 800ms
Type "?"
Wait+Screen "Open and inspect:"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/help-overlays-log.png
State target/dogfood-artifacts/betamax/help-overlays-log.json

Caption "Diff help groups file movement, folding, search, and return controls."
Type "d"
Wait+Screen "jk jj diff"
Sleep 600ms
Type "?"
Wait+Screen "Move and find:"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/help-overlays-diff.png
State target/dogfood-artifacts/betamax/help-overlays-diff.json
Escape
Sleep 300ms

Caption "Workspace help keeps workspace-specific actions together."
Type "W"
Wait+Screen "jk jj workspace"
Sleep 600ms
Type "?"
Wait+Screen "Open log"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/help-overlays-workspaces.png
State target/dogfood-artifacts/betamax/help-overlays-workspaces.json
Escape
Sleep 300ms

Caption "Command history help centers the retained command workflow."
Type "C"
Wait+Screen "Command History"
Sleep 600ms
Type "?"
Wait+Screen "Open command details"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/help-overlays-command-history.png
State target/dogfood-artifacts/betamax/help-overlays-command-history.json
Escape
Sleep 300ms
Escape
Sleep 300ms
Type "L"
Wait+Screen "jk jj"
Sleep 300ms

Caption "Operation log help keeps recovery and operation inspection visible."
Type "o"
Wait+Screen "op log"
Sleep 600ms
Type "?"
Wait+Screen "Open show"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/help-overlays-operation-log.png
State target/dogfood-artifacts/betamax/help-overlays-operation-log.json

Caption ""
Escape
Sleep 300ms
Type "q"
Hide
Type "exit"
Enter
