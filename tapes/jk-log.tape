Output target/dogfood-artifacts/betamax/jk-log.gif
Output target/dogfood-artifacts/betamax/jk-log-final-state.json

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
Type "repo=${JK_REPO:-$HOME/local/jk/default}"
Enter
Wait
Type "cd \"$repo\" && test -f Cargo.toml && test -d .jj && pwd"
Enter
Wait+Screen "/local/jk"
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-betamax-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Show

Caption "The home log starts with the working-copy change selected."
Type "./target/debug/jk -R . -n 3"
Enter
Wait+Screen "jk jj"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/jk-log-initial.png
State target/dogfood-artifacts/betamax/jk-log-initial.json

Caption "L switches to the explicit jj log root view."
Type "L"
Wait+Screen "jk jj log"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-log-switched-log.png
State target/dogfood-artifacts/betamax/jk-log-switched-log.json

Caption "H returns to the home view without losing the selected change."
Type "H"
Wait+Screen "/jk jj *\\n/"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-log-switched-home.png
State target/dogfood-artifacts/betamax/jk-log-switched-home.json

Caption "j and k move the selected change through the graph."
Type "j"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/jk-log-selected-second.png
State target/dogfood-artifacts/betamax/jk-log-selected-second.json

Type "k"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/jk-log-selected-first-again.png
State target/dogfood-artifacts/betamax/jk-log-selected-first-again.json

Caption "Enter opens the selected change without replacing the log context."
Enter
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-log-expanded.png
State target/dogfood-artifacts/betamax/jk-log-expanded.json

Caption "Arrow keys collapse and expand long rendered change details."
Left
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/jk-log-collapsed-left.png
State target/dogfood-artifacts/betamax/jk-log-collapsed-left.json

Right
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/jk-log-expanded-right.png
State target/dogfood-artifacts/betamax/jk-log-expanded-right.json

Caption "Refresh re-runs the jj command while preserving the visible context."
Type "r"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-log-refreshed.png
State target/dogfood-artifacts/betamax/jk-log-refreshed.json

Caption ""
Type "q"
Sleep 300ms

Caption "Direct root commands open the same jk surface."
Type "./target/debug/jk -R . log -n 3"
Enter
Wait+Screen "jk jj log"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/jk-log-command.png
State target/dogfood-artifacts/betamax/jk-log-command.json

Caption ""
Type "q"
Hide
Type "exit"
Enter
