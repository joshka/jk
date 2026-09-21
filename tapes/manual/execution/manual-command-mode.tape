Output target/manual/manual-command-mode.mp4
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1400
Set Height 900
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 80ms
Set WaitTimeout 45s
Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" changes"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\"  -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 2s
Caption "Colon runs jj with explicit arguments; an unmatched quote is corrected before execution."
Type ":"
Type "status '"
Enter
Wait+Screen "unterminated single quote"
Sleep 1s
Screenshot target/manual/manual-command-mode-parse-error.png
State target/manual/manual-command-mode-parse-error.json
Sleep 2s
Ctrl+u
Type "status"
Enter
Wait+Screen "Status: success"
Sleep 1s
Screenshot target/manual/manual-command-mode-jj-success.png
State target/manual/manual-command-mode-jj-success.json
Sleep 2s
Caption "Backspace returns from captured output. Escape cancels a draft command before execution."
Backspace
Wait+Screen "Experiment with forecast caching"
Type ":describe -m discarded"
Sleep 2s
Escape
Wait+Screen "Experiment with forecast caching"
Caption "External commands use exact argv; an explicit shell is needed for shell syntax."
Type "!"
Type "printf '%s\\n' 'Cache verification complete'"
Enter
Wait+Screen "Cache verification complete"
Wait+Screen "Status: success"
Sleep 3s
Type "!"
Type "sh -c 'printf \"cache check failed\\n\" >&2; exit 7'"
Enter
Wait+Screen "Status: exit 7"
Wait+Screen "cache check failed"
Sleep 1s
Screenshot target/manual/manual-command-mode-failure.png
State target/manual/manual-command-mode-failure.json
Sleep 2s
Caption "Command History keeps failed output so diagnostics remain available."
Type "C"
Wait+Screen "Command History"
Sleep 1s
Screenshot target/manual/manual-command-mode-history.png
State target/manual/manual-command-mode-history.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-command-mode"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-command-mode"
Type "exit"
Enter
