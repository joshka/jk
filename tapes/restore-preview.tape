Output target/dogfood-artifacts/betamax/restore-preview.gif
Output target/dogfood-artifacts/betamax/restore-preview-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1400
Set Height 900
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
Type "test -f /Users/joshka/local/jk/work/restore-workflow/Cargo.toml && mkdir -p /Users/joshka/local/jk/work/restore-workflow/target/dogfood-artifacts/betamax"
Enter
Wait
Type "cd /Users/joshka/local/jk/work/restore-workflow && cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-restore-preview-build.log 2>&1"
Enter
Wait
Type "rm -rf /tmp/jk-restore-preview-fixture"
Enter
Wait
Type "mkdir -p /tmp/jk-restore-preview-fixture && cd /tmp/jk-restore-preview-fixture && jj git init --colocate"
Enter
Wait
Type "printf 'base content\\n' > /tmp/jk-restore-preview-fixture/file.txt && cd /tmp/jk-restore-preview-fixture && jj file track file.txt && jj commit -m 'Base content'"
Enter
Wait
Type "printf 'working content\\n' > /tmp/jk-restore-preview-fixture/file.txt && cd /tmp/jk-restore-preview-fixture && jj describe -m 'Working content'"
Enter
Wait
Show

Caption "Choose one source revision, then open Restore from the action menu."
Type "/Users/joshka/local/jk/work/restore-workflow/target/debug/jk -R /tmp/jk-restore-preview-fixture -n 3"
Enter
Wait+Screen "Working content"
Type "j"
Sleep 300ms
Type "a"
Wait+Screen "Actions"
Type "r"
Wait+Screen "Affected content: all paths"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-00-confirm.png
State target/dogfood-artifacts/betamax/restore-preview-00-confirm.json

Caption "Escape cancels without changing the repository or losing log context."
Escape
Wait+Screen "Base content"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-01-cancelled.png
State target/dogfood-artifacts/betamax/restore-preview-01-cancelled.json

Caption "Confirming runs the displayed argv, refreshes the log, and exposes recovery."
Type "a"
Wait+Screen "Actions"
Type "r"
Wait+Screen "Affected content: all paths"
Enter
Wait+Screen "Restored all paths into working copy"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-02-success.png
State target/dogfood-artifacts/betamax/restore-preview-02-success.json

Caption "Command History retains the exact restore and resulting operation."
Type "C"
Wait+Screen "Command History"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/restore-preview-03-history.png
State target/dogfood-artifacts/betamax/restore-preview-03-history.json
Type "q"
Hide
Type "exit"
Enter
