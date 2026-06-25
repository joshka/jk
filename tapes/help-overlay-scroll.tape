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
Type "repo=${JK_REPO:-/Users/joshka/local/jk/help-overlays}"
Enter
Wait
Type "cd \"$repo\" && test -f Cargo.toml && test -d .jj && echo JK_REPO_READY"
Enter
Wait+Screen "JK_REPO_READY"
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-help-overlay-scroll-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Show

Caption "Help opens as a contextual reference."
Type "./target/debug/jk -R . -n 8"
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
