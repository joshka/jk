Output ../../jk-screenshots/assets/jk-diff.gif
Output target/betamax/readme-diff-final-state.json

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

Type "./target/debug/jk -R . diff ommyvtpx"
Enter
Wait+Screen "jk jj diff"
Sleep 700ms
Screenshot ../../jk-screenshots/assets/jk-diff.png

Type "?"
Sleep 500ms
Screenshot ../../jk-screenshots/assets/jk-diff-help.png

Type "?"
Sleep 300ms
Type "/"
Sleep 150ms
Type "diff"
Enter
Sleep 400ms
Screenshot ../../jk-screenshots/assets/jk-diff-search.png

Type "}"
Sleep 300ms
Type "-"
Sleep 500ms
Screenshot ../../jk-screenshots/assets/jk-diff-folded-hunk.png

Type "+"
Sleep 250ms
Type "h"
Sleep 500ms
Screenshot ../../jk-screenshots/assets/jk-diff-folded-file.png

Type "q"
Sleep 400ms
Hide
Type "exit"
Enter
