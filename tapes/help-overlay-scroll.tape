Output target/dogfood-artifacts/betamax/help-overlay-scroll.gif
Output target/dogfood-artifacts/betamax/help-overlay-scroll-final-state.json

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

Caption "Help opens as a contextual reference."
Type "\"$bin\" -R \"$fixture\" -n 8"
Enter
Wait+Screen "jk jj"
Sleep 800ms
Type "?"
Wait+Screen "Open and inspect:"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-00-open.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-00-open.json

Caption "Down should scroll the help content predictably."
Down
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-01-down.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-01-down.json
Down
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-02-down.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-02-down.json
Down
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-03-down.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-03-down.json

Caption "Up should reverse that scroll without jumping columns."
Up
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-04-up.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-04-up.json
Up
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-05-up.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-05-up.json

Caption ""
Escape
Sleep 300ms
Type "q"
Hide
Type "exit"
Enter
