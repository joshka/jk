Output target/dogfood-artifacts/betamax/external-command-mode-20260919.gif
Output target/dogfood-artifacts/betamax/external-command-mode-final-state.json

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
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 35ms
Set WaitTimeout 60s
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
Type "test -x \"$bin\" && printf 'JK_%s\\n' BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Type "fixture=$(mktemp -d) && cd \"$fixture\" && jj git init --colocate >/dev/null 2>&1"
Enter
Wait
Type "jj describe -m 'External command demo' >/dev/null 2>&1"
Enter
Wait
Type "clear && \"$bin\" -R \"$fixture\" -n 5"
Enter
Wait+Screen "External command demo"
Show
Sleep 1s

Caption "! runs an executable with explicit argv and captures its output."
Type "!"
Type "printf '%s\\n' 'hello from jk'"
Enter
Wait+Screen "hello from jk"
Wait+Screen "Status: success"
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-success-20260919.png
State target/dogfood-artifacts/betamax/external-command-success-20260919.json

Caption "Shell syntax only runs when the command explicitly requests sh -c."
Type "!"
Type "sh -c 'printf \"captured stderr\\n\" >&2; exit 7'"
Enter
Wait+Screen "Status: exit 7"
Wait+Screen "captured stderr"
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-failure-20260919.png
State target/dogfood-artifacts/betamax/external-command-failure-20260919.json

Caption "Failures remain available in Command History with external identity and status."
Type "C"
Wait+Screen "Command History"
Wait+Screen "external command"
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-history-20260919.png
State target/dogfood-artifacts/betamax/external-command-history-20260919.json

Caption "Contextual discovery documents ! as the external-command entry point."
Type "?"
Sleep 1s
Type "jjjjjjjjjjjj"
Sleep 1s
Sleep 2s
Screenshot target/dogfood-artifacts/betamax/external-command-help-20260919.png
State target/dogfood-artifacts/betamax/external-command-help-20260919.json
State target/dogfood-artifacts/betamax/external-command-mode-final-state.json

Caption ""
Type "?"
Sleep 300ms
Type "q"
