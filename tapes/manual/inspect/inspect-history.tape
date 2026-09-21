# Manual scenario: inspect-history
Output target/manual/inspect-history.mp4
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1400
Set Height 840
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 70ms
Set WaitTimeout 30s

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" inspection"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show

Caption "v shows earlier versions of this change, including description rewrites."
Type "v"

Caption "Evolution follows one change through rewrites; it is different from the repository operation log."
Wait+Screen "jk jj evolog"
Sleep 2500ms
Screenshot target/manual/inspect-history-evolog.png
State target/manual/inspect-history-evolog.json

Type "L"
Wait+Screen "Experiment with forecast caching"
Caption "Command mode can run the same read-only evolog command with explicit operands."
Type ":"
Type "evolog -r @ --no-graph -n 3"
Enter
Wait+Screen "Status: success"
Sleep 2s
Caption "Interdiff compares the patches introduced by two revisions."
Type ":"
Type "interdiff --from missing-review --to @"
Enter

Caption "A misspelled revision produces retained stderr and a failed status; the repository is unchanged."
Wait+Screen "Status: exit"
Sleep 2500ms
Screenshot target/manual/inspect-history-failure.png
State target/manual/inspect-history-failure.json

Caption "e reopens the failed input. Ctrl-u clears it so you can enter the intended revisions."
Type "e"
Ctrl+u
Type "interdiff --from @- --to @"
Enter

Caption "The corrected interdiff shows how the two changes differ. Command History keeps both attempts."
Wait+Screen "Status: success"
Sleep 2500ms
Screenshot target/manual/inspect-history-interdiff.png
State target/manual/inspect-history-interdiff.json

Caption "C keeps both the failure and the successful retry available during this session."
Type "C"

Caption "Inspect command history to verify the command, exit status, output, and operation metadata."
Wait+Screen "Command History"
Sleep 2500ms
Screenshot target/manual/inspect-history-history.png
State target/manual/inspect-history-history.json

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-history"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
