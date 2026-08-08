Output target/dogfood-artifacts/betamax/squash-workflow.gif
Output target/dogfood-artifacts/betamax/squash-workflow-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
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
Type "source_repo=${JK_SOURCE_REPO:-$HOME/local/jk/work/squash-workflow}"
Enter
Wait
Type "cd \"$source_repo\" && test -f Cargo.toml && mkdir -p target/dogfood-artifacts/betamax && echo JK_SOURCE_READY"
Enter
Wait+Screen "JK_SOURCE_READY"
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-squash-workflow-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Type "fixture=$(mktemp -d)"
Enter
Wait
Type "cd \"$fixture\" && jj git init --colocate >/tmp/jk-squash-workflow-init.log 2>&1"
Enter
Wait
Type "printf 'destination contents\n' > destination.txt && jj file track destination.txt >/dev/null 2>&1 || true"
Enter
Wait
Type "jj describe -m 'Destination change' >/dev/null && jj new -m 'Source change' >/dev/null"
Enter
Wait
Type "printf 'source contents\n' > source.txt && bin=\"$source_repo/target/debug/jk\" && echo JK_FIXTURE_READY"
Enter
Wait+Screen "JK_FIXTURE_READY"
Show

Caption "Mark the source, move to the destination, then choose Squash from Actions."
Type "\"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "Source change"
Sleep 700ms
Space
Sleep 500ms
Type "j"
Sleep 500ms
Type "a"
Wait+Screen "Squash"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-action-menu.png
Type "s"
Wait+Screen "Roles and scope"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-preview.png
State target/dogfood-artifacts/betamax/squash-workflow-preview.json

Caption "Esc cancels without losing the destination cursor or ordered source mark."
Escape
Wait+Screen "Destination change"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-cancelled.png

Caption "Confirm the same exact whole-change command."
Type "a"
Type "s"
Wait+Screen "Roles and scope"
Sleep 800ms
Enter
Wait+Screen "Squashed source changes"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-complete.png
State target/dogfood-artifacts/betamax/squash-workflow-complete.json

Caption "Command History records the squash and its resulting operation."
Type "C"
Wait+Screen "Command History"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-history.png
State target/dogfood-artifacts/betamax/squash-workflow-history.json

Caption "Open the captured operation for audit and recovery context."
Type "o"
Wait+Screen "Changed commits"
Sleep 1200ms
Screenshot target/dogfood-artifacts/betamax/squash-workflow-operation.png
State target/dogfood-artifacts/betamax/squash-workflow-operation.json

Caption ""
Type "q"
Hide
Type "exit"
Enter
