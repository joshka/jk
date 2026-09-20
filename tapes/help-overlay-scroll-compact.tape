Output target/dogfood-artifacts/betamax/help-overlay-scroll-compact.gif
Output target/dogfood-artifacts/betamax/help-overlay-scroll-compact-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 720
Set Height 640
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

Caption "Compact terminal: Help remains a scrolling reference."
Type "\"$bin\" -R \"$fixture\" -n 8"
Enter
Wait+Screen "jk jj"
Sleep 800ms
Type "?"
Wait+Screen "Open and inspect:"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-compact-00-open.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-compact-00-open.json

Caption "A single Down moves the help body up by one rendered line."
Down
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-compact-01-down.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-compact-01-down.json
Down
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-compact-02-down.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-compact-02-down.json

Caption ""
Escape
Sleep 300ms
Type "q"
Hide
Type "exit"
Enter
