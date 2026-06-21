Output ../../jk-screenshots/assets/jk-log-v3.gif
Output target/betamax/readme-log-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set Width 1200
Set Height 720
Set Padding 0
Set Margin 0
Set WindowBar Colorful
Set BorderRadius 0
Set CursorBlink false
Set TypingSpeed 24ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type "repo=${JK_REPO:-$HOME/local/jk/default}"
Enter
Wait
Type "cd \"$repo\" && test -f Cargo.toml && test -d .jj && mkdir -p ../../jk-screenshots/assets target/betamax"
Enter
Wait+Line
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-readme-betamax-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Ctrl+L
Sleep 200ms

Type "./target/debug/jk -R . -n 8"
Enter
Wait+Screen "jk jj"
Sleep 1s
Show
Screenshot ../../jk-screenshots/assets/jk-log-v3.png

Hide
Type "?"
Sleep 500ms
Screenshot ../../jk-screenshots/assets/jk-log-help-v3.png

Type "?"
Sleep 300ms
Show
Type "j"
Sleep 450ms
Type "j"
Sleep 700ms
Screenshot ../../jk-screenshots/assets/jk-log-selection-v3.png

Enter
Sleep 700ms
Screenshot ../../jk-screenshots/assets/jk-log-expanded-v3.png

Type "d"
Wait+Screen "jk jj diff"
Sleep 1200ms
Type "}"
Sleep 700ms
Type "-"
Sleep 1s
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 900ms
Type "+"
Sleep 800ms
Type "H"
Wait+Screen "jk jj"
Sleep 900ms
Hide
Type "q"
Sleep 400ms
Type "exit"
Enter
