Output ../../jk-screenshots/assets/jk-log.gif
Output target/betamax/readme-log-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1200
Set Height 720
Set Margin 12
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set TypingSpeed 18ms
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
Show

Type "./target/debug/jk -R . -n 8"
Enter
Wait+Screen "jk jj"
Sleep 700ms
Screenshot ../../jk-screenshots/assets/jk-log.png

Type "?"
Sleep 500ms
Screenshot ../../jk-screenshots/assets/jk-log-help.png

Type "?"
Sleep 300ms
Type "j"
Sleep 250ms
Type "j"
Sleep 250ms
Screenshot ../../jk-screenshots/assets/jk-log-selection.png

Enter
Sleep 500ms
Screenshot ../../jk-screenshots/assets/jk-log-expanded.png

Type "q"
Sleep 300ms
Hide
Type "exit"
Enter
