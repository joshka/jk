Output target/manual/manual-invalid-operands.mp4
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" run-options"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Rebase source"
Show
Sleep 2s
Caption "Two marks are ambiguous for rebase. Keep one source marked before choosing a destination."
Type " "
Type "j"
Type " "
Type "R"
Wait+Screen "Rebase source is ambiguous"
Sleep 1s
Screenshot target/manual/manual-invalid-operands-rebase.png
State target/manual/manual-invalid-operands-rebase.json
Sleep 2s
Caption "Restore operates on one cursor revision and refuses active marks."
Type "ar"
Wait+Screen "Restore"
Sleep 1s
Screenshot target/manual/manual-invalid-operands-restore.png
State target/manual/manual-invalid-operands-restore.json
Sleep 2s
Type "c"
Caption "Squash requires marked source changes and a different destination cursor."
Type "as"
Wait+Screen "Mark at least one source"
Sleep 1s
Screenshot target/manual/manual-invalid-operands-squash.png
State target/manual/manual-invalid-operands-squash.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-invalid-operands"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-invalid-operands"
Type "exit"
Enter
