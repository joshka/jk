Output target/dogfood-artifacts/betamax/help-overlay-scroll-wide.gif
Output target/dogfood-artifacts/betamax/help-overlay-scroll-wide-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 2850
Set Height 1250
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
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-help-overlay-scroll-wide-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Show

Caption "Wide terminal around 200x50 cells: Help uses aligned columns."
Type "./target/debug/jk -R . -n 8"
Enter
Wait+Screen "jk jj"
Sleep 800ms
Type "?"
Wait+Screen "Open and inspect:"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-wide-00-open.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-wide-00-open.json

Caption "Down leaves the fitted help stable when every command is visible."
Down
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/help-overlay-scroll-wide-01-down.png
State target/dogfood-artifacts/betamax/help-overlay-scroll-wide-01-down.json

Caption ""
Escape
Sleep 300ms
Type "q"
Hide
Type "exit"
Enter
