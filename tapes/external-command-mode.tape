Output target/dogfood-artifacts/betamax/external-command-mode-20260919.gif
Output target/dogfood-artifacts/betamax/external-command-mode-final-state.json

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
Set TypingSpeed 35ms
Set WaitTimeout 60s
Env NO_COLOR "1"

Hide
Type "source_repo=/Users/joshka/local/jk/work/external-command-mode"
Enter
Wait
Type "cd \"$source_repo\" && mkdir -p target/dogfood-artifacts/betamax"
Enter
Wait
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-external-command-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Type "fixture=$(mktemp -d) && cd \"$fixture\" && jj git init --colocate >/dev/null 2>&1"
Enter
Wait
Type "jj describe -m 'External command demo' >/dev/null && bin=\"$source_repo/target/debug/jk\""
Enter
Wait
Type "clear && exec \"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "External command demo"
Show
Sleep 1s

Caption "! runs an executable with explicit argv and captures its output."
Type "!"
Type "printf '%s\\n' 'hello from jk'"
Enter
Wait+Screen "hello from jk"
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-success-20260919.png

Caption "Shell syntax only runs when the command explicitly requests sh -c."
Type "!"
Type "sh -c 'printf \"captured stderr\\n\" >&2; exit 7'"
Enter
Wait+Screen "Status: exit 7"
Wait+Screen "captured stderr"
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-failure-20260919.png

Caption "Failures remain available in Command History with external identity and status."
Type "C"
Wait+Screen "Command History"
Wait+Screen "external command"
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-history-20260919.png

Caption "Contextual discovery documents ! as the external-command entry point."
Type "?"
Sleep 1s
Type "jjjjjjjjjjjj"
Sleep 1s
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-help-20260919.png
State target/dogfood-artifacts/betamax/external-command-mode-final-state.json

Caption ""
Type "?"
Sleep 300ms
Type "q"
