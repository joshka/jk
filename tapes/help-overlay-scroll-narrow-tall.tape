Output target/dogfood-artifacts/betamax/help-overlay-scroll-narrow-tall.gif
Output target/dogfood-artifacts/betamax/help-overlay-scroll-narrow-tall-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 720
Set Height 1050
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" inspection"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Show

Caption "Narrow but tall terminal: Help keeps a stable scrolling viewport."
Type "\"$bin\" -R \"$fixture\" -n 8"
Enter
Wait+Screen "jk jj"
Sleep 800ms
Type "?"
Wait+Screen "Open and inspect:"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-narrow-tall-00-open.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-narrow-tall-00-open.json

Caption "Down still scrolls one line instead of expanding to all content."
Down
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-narrow-tall-01-down.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-narrow-tall-01-down.json

Caption ""
Escape
Sleep 300ms
Type "q"
Hide
Type "exit"
Enter
