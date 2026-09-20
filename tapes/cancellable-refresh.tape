Output target/dogfood-artifacts/betamax/cancellable-refresh-20260919.gif
Output target/dogfood-artifacts/betamax/cancellable-refresh-final-state.json

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
Env JJ_CONFIG "/dev/null"
Env JJ_USER "Demo Author"
Env JJ_EMAIL "demo@example.invalid"
Env JJ_TIMESTAMP "2026-01-01T12:00:00Z"
Env JJ_OP_TIMESTAMP "2026-01-01T12:00:00Z"

Hide
Type "source_repo=${JK_SOURCE_REPO:-$PWD}; bin=${JK_BIN:-$source_repo/target/debug/jk}"
Enter
Wait
Type "test -x \"$bin\" && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Type "fixture=$(mktemp -d) && cd \"$fixture\" && jj git init --colocate >/dev/null 2>&1"
Enter
Wait
Type "jj describe -m 'Base change' >/dev/null 2>&1 && jj new -m 'Refresh demo' >/dev/null 2>&1"
Enter
Wait
Type "mkdir fake-bin state && ln -s \"$source_repo/scripts/betamax-cancellable-jj.sh\" fake-bin/jj"
Enter
Wait
Type "export JK_REAL_JJ=$(command -v jj) JK_FAKE_JJ_STATE=\"$fixture/state\" PATH=\"$fixture/fake-bin:$PATH\""
Enter
Wait
Type "clear && \"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "Refresh demo"
Show

Caption "A slow refresh keeps the current log visible."
Type "r"
Wait+Screen "Refreshing…"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-00-loading.png
State target/dogfood-artifacts/betamax/cancellable-refresh-00-loading.json

Caption "Navigation remains responsive while jj is still running."
Type "j"
Wait+Screen "Refreshing…"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-01-navigation.png
State target/dogfood-artifacts/betamax/cancellable-refresh-01-navigation.json

Caption "A newer refresh cancels and supersedes the old generation."
Type "r"
Wait+Screen "Refreshing…"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-02-complete.png
State target/dogfood-artifacts/betamax/cancellable-refresh-02-complete.json

Caption "Failure keeps the last usable log and reports what failed."
Type "r"
Wait+Screen "Refresh failed:"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-03-failure.png
State target/dogfood-artifacts/betamax/cancellable-refresh-03-failure.json

Caption ""
Type "q"
Sleep 300ms
Hide
Type "exit"
Enter
