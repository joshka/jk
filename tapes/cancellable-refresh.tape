Output target/dogfood-artifacts/betamax/cancellable-refresh-20260919.gif
Output target/dogfood-artifacts/betamax/cancellable-refresh-final-state.json

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
Type "repo=/Users/joshka/local/jk/work/cancellable-refresh"
Enter
Wait
Type "cd \"$repo\" && test -f Cargo.toml && test -d .jj"
Enter
Wait
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-cancellable-refresh-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Type "mkdir -p target/dogfood-artifacts/betamax/fake-bin target/dogfood-artifacts/betamax/cancellable-state"
Enter
Wait
Type "rm -f target/dogfood-artifacts/betamax/cancellable-state/count && real_jj=$(command -v jj)"
Enter
Wait
Type "ln -sf \"$PWD/scripts/betamax-cancellable-jj.sh\" target/dogfood-artifacts/betamax/fake-bin/jj && chmod +x scripts/betamax-cancellable-jj.sh"
Enter
Wait
Type "export JK_REAL_JJ=\"$real_jj\" JK_FAKE_JJ_STATE=\"$PWD/target/dogfood-artifacts/betamax/cancellable-state\" PATH=\"$PWD/target/dogfood-artifacts/betamax/fake-bin:$PATH\""
Enter
Wait
Show

Caption "A slow refresh keeps the current log visible."
Type "./target/debug/jk -R . -n 5"
Enter
Wait+Screen "jk jj"
Type "r"
Wait+Screen "Refreshing…"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-00-loading.png

Caption "Navigation remains responsive while jj is still running."
Type "j"
Wait+Screen "Refreshing…"
Sleep 700ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-01-navigation.png

Caption "A newer refresh cancels and supersedes the old generation."
Type "r"
Wait+Screen "Refreshing…"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-02-complete.png

Caption "Failure keeps the last usable log and reports what failed."
Type "r"
Wait+Screen "Refresh failed:"
Sleep 1000ms
Screenshot target/dogfood-artifacts/betamax/cancellable-refresh-03-failure.png

Caption ""
Type "q"
Sleep 300ms
Hide
Type "exit"
Enter
