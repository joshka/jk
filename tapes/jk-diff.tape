Output target/dogfood-artifacts/betamax/jk-diff.gif
Output target/dogfood-artifacts/betamax/jk-diff-final-state.json

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

Caption "Open a diff from the log when you want to inspect the selected change."
Type "./target/debug/jk -R . -n 8"
Enter
Wait+Screen "jk jj"
Sleep 500ms
Type "d"
Wait+Screen "jk jj diff"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-open.png
State target/dogfood-artifacts/betamax/jk-diff-open.json

Caption "Help is available without leaving the diff view."
Type "?"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-help.png
State target/dogfood-artifacts/betamax/jk-diff-help.json

Type "?"
Sleep 300ms
Caption "Search highlights matching diff text in place."
Type "/"
Sleep 200ms
Type "diff"
Enter
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-search.png
State target/dogfood-artifacts/betamax/jk-diff-search.json

Caption "Jump between hunks instead of paging blindly through long patches."
Type "}"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-next-hunk.png
State target/dogfood-artifacts/betamax/jk-diff-next-hunk.json

Caption "Fold a hunk when its context is no longer relevant."
Type "-"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-fold-hunk.png
State target/dogfood-artifacts/betamax/jk-diff-fold-hunk.json

Caption "Fold a file to reduce a large diff to the parts still in play."
Type "+"
Sleep 300ms
Type "h"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-fold-file.png
State target/dogfood-artifacts/betamax/jk-diff-fold-file.json

Caption "Horizontal movement keeps long diff lines readable."
Type "l"
Sleep 300ms
Type ">"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-horizontal.png
State target/dogfood-artifacts/betamax/jk-diff-horizontal.json

Caption "Return to the log when the inspection is done."
Type "L"
Wait+Screen "jk jj"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/jk-diff-return-log.png
State target/dogfood-artifacts/betamax/jk-diff-return-log.json

Caption ""
Type "q"
Hide
Type "exit"
Enter
