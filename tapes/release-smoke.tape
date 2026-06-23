Output target/dogfood-artifacts/betamax/release-smoke.gif
Output target/dogfood-artifacts/betamax/release-smoke-final-state.json

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
Type "source_repo=${JK_SOURCE_REPO:-$HOME/local/jk/default}"
Enter
Wait
Type "cd \"$source_repo\" && test -f Cargo.toml && mkdir -p target/dogfood-artifacts/betamax && echo JK_SOURCE_READY"
Enter
Wait+Screen "JK_SOURCE_READY"
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-release-smoke-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Type "fixture=$(mktemp -d)"
Enter
Wait
Type "cd \"$fixture\" && jj git init --colocate >/tmp/jk-release-smoke-init.log 2>&1"
Enter
Wait
Type "printf 'one\\n' > file.txt && jj file track file.txt >/dev/null 2>&1 || true"
Enter
Wait
Type "jj describe -m 'Initial workspace' >/dev/null && jj new -m 'Second change' >/dev/null"
Enter
Wait
Type "printf 'two\\n' >> file.txt && bin=\"$source_repo/target/debug/jk\" && echo JK_FIXTURE_READY"
Enter
Wait+Screen "JK_FIXTURE_READY"
Show

Caption "Root status opens the same rendered jk surface used inside jk."
Type "\"$bin\" -R \"$fixture\" status"
Enter
Wait+Screen "file.txt"
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-status.json
Type "q"
Sleep 1500ms

Caption "Workspace scope is available as an early, top-level workflow."
Type "\"$bin\" -R \"$fixture\" workspaces"
Enter
Wait+Screen "workspace list"
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-workspaces.json
Type "q"
Sleep 1500ms

Caption "Command mode accepts direct jj commands without leaving the TUI."
Type "\"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "jk jj"
Sleep 500ms
Type ":"
Sleep 200ms
Type "status"
Enter
Wait+Screen "file.txt"
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-command-status.json
Type "q"
Sleep 1500ms

Caption "Failed jj commands stay visible as recoverable command output."
Type "\"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "jk jj"
Sleep 500ms
Type ":"
Sleep 200ms
Type "not-a-jj-command"
Enter
Wait+Screen "not-a-jj-command"
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-command-failure.json
Type "q"
Sleep 1500ms

Caption "Mutation shortcuts show a jj command preview before anything runs."
Type "\"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "jk jj"
Sleep 500ms
Type "m"
Wait+Screen "Describe revision"
Ctrl+U
Type "Release smoke message"
Enter
Wait+Screen "Confirm command"
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-describe-preview.json
Enter
Wait+Screen "Release smoke message"
Caption "After confirmation, jk refreshes the log with the new description."
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-after-describe.json
Caption "Command History records the mutation, output, and captured operation."
Type "C"
Wait+Screen "Command History"
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-command-history.json
Caption "The captured operation opens directly for recovery and audit context."
Type "o"
Wait+Screen "Changed commits"
Sleep 1200ms
State target/dogfood-artifacts/betamax/release-smoke-operation-show.json
Caption ""
Type "q"
Hide
Type "exit"
Enter
