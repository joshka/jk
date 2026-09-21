# Manual scenario: inspect-operations
Output target/manual/inspect-operations.mp4
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

Caption "o opens repository operations: snapshots of source-control actions, not source revisions."
Type "o"

Caption "Choose an operation to understand what changed in repository metadata and history."
Wait+Screen "jk jj op log"
Sleep 2500ms
Screenshot target/manual/inspect-operations-log.png
State target/manual/inspect-operations-log.json

Caption "Enter inspects the selected operation without restoring it."
Enter

Caption "Operation show records the action and affected changes. Read it before considering recovery."
Wait+Screen "jk jj op show"
Sleep 2500ms
Screenshot target/manual/inspect-operations-show.png
State target/manual/inspect-operations-show.json

Type "/"
Type "candidate"
Enter
Sleep 2s
Type "L"
Wait+Screen "jk jj op log"
Caption "d compares the selected operation with its parent operation."
Type "d"

Caption "Operation diff makes the recorded repository transition inspectable."
Wait+Screen "jk jj op diff"
Sleep 2500ms
Screenshot target/manual/inspect-operations-diff.png
State target/manual/inspect-operations-diff.json

Type "L"
Wait+Screen "jk jj op log"
Type "r"
Wait+Screen "jk jj op log"
Sleep 2s
Escape

Caption "Escape leaves operation history and returns to the source graph without undoing anything."
Wait+Screen "Experiment with forecast caching"
Sleep 2500ms
Screenshot target/manual/inspect-operations-returned.png
State target/manual/inspect-operations-returned.json

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-operations"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
