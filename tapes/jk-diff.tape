Output target/betamax/jk-diff.gif
Output target/betamax/jk-diff-final-state.json

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

Type "./target/debug/jk -R . -n 8"
Enter
Wait+Screen "jk jj"
Sleep 500ms
Type "d"
Wait+Screen "jk jj diff"
Sleep 500ms
Screenshot target/betamax/jk-diff-open.png
State target/betamax/jk-diff-open.json

Type "?"
Sleep 300ms
Screenshot target/betamax/jk-diff-help.png
State target/betamax/jk-diff-help.json

Type "?"
Sleep 300ms
Type "/"
Sleep 200ms
Type "diff"
Enter
Sleep 300ms
Screenshot target/betamax/jk-diff-search.png
State target/betamax/jk-diff-search.json

Type "}"
Sleep 300ms
Screenshot target/betamax/jk-diff-next-hunk.png
State target/betamax/jk-diff-next-hunk.json

Type "-"
Sleep 300ms
Screenshot target/betamax/jk-diff-fold-hunk.png
State target/betamax/jk-diff-fold-hunk.json

Type "+"
Sleep 300ms
Type "h"
Sleep 300ms
Screenshot target/betamax/jk-diff-fold-file.png
State target/betamax/jk-diff-fold-file.json

Type "l"
Sleep 300ms
Type ">"
Sleep 300ms
Screenshot target/betamax/jk-diff-horizontal.png
State target/betamax/jk-diff-horizontal.json

Type "L"
Wait+Screen "jk jj"
Sleep 300ms
Screenshot target/betamax/jk-diff-return-log.png
State target/betamax/jk-diff-return-log.json

Type "q"
Hide
Type "exit"
Enter
