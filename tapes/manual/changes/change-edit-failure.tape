Output target/manual/change-edit-failure.mp4

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
Set WaitTimeout 60s

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" changes"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 3s
Caption "The root revision cannot become a working copy. A failed edit leaves the graph intact."
End
Sleep 2s
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "e"
Wait+Screen "jj edit failed:"
Sleep 3s
Screenshot target/manual/change-edit-failure-error.png
State target/manual/change-edit-failure-error.json
Caption "Command History retains the failed command and the complete jj diagnostic."
Type "C"
Wait+Screen "Command History"
Sleep 2s
Home
Enter
Wait+Screen "Status: exit 1"
Sleep 3s
Screenshot target/manual/change-edit-failure-history.png
State target/manual/change-edit-failure-history.json
Caption "Return to the log and choose an editable change to continue."
Backspace
Wait+Screen "Command History"
Sleep 1s
Backspace
Wait+Screen "Experiment with forecast caching"
Home
Sleep 3s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" change-edit-failure"
Enter
Wait+Screen "MANUAL_ASSERT_OK change-edit-failure"
