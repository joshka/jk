Output target/manual/manual-squash.mp4
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" squash"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Squash source"
Show
Sleep 2s
Caption "Mark the source, move to its destination, and review the whole-change squash."
Type " "
Type "j"
Type "as"
Wait+Screen "Source 1:"
Sleep 1s
Screenshot target/manual/manual-squash-preview.png
State target/manual/manual-squash-preview.json
Sleep 2s
Escape
Wait+Screen "Squash destination"
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-squash-cancelled"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-squash-cancelled"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Squash source"
Show
Sleep 2s
Type " "
Type "j"
Type "as"
Wait+Screen "Source 1:"
Sleep 2s
Enter
Wait+Screen "Squashed source changes"
Sleep 1s
Screenshot target/manual/manual-squash-success.png
State target/manual/manual-squash-success.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-squash-success"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-squash-success"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Squash destination"
Show
Sleep 2s
Caption "Undo restores the prior repository content; verify the files before continuing."
Type "au"
Wait+Screen "Undid operation"
Sleep 1s
Screenshot target/manual/manual-squash-recovered.png
State target/manual/manual-squash-recovered.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-squash-undone"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-squash-undone"
Type "exit"
Enter
