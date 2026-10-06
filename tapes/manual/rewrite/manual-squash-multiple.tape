Output target/manual/manual-squash-multiple.mp4
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" squash-multiple"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Squash second source"
Show
Sleep 2s
Caption "Mark both sources in order, move to the destination, and review the whole-change squash."
Type " "
Type "j"
Type " "
Type "j"
Type "as"
Wait+Screen "Source 2:"
Sleep 1s
Screenshot target/manual/manual-squash-multiple-preview.png
State target/manual/manual-squash-multiple-preview.json
Sleep 2s
Escape
Wait+Screen "Squash destination"
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-squash-multiple-cancelled"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-squash-multiple-cancelled"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Squash second source"
Show
Sleep 2s
Type " "
Type "j"
Type " "
Type "j"
Type "as"
Wait+Screen "Source 2:"
Sleep 2s
Enter
Wait+Screen "Squashed source changes"
Sleep 1s
Screenshot target/manual/manual-squash-multiple-success.png
State target/manual/manual-squash-multiple-success.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-squash-multiple-success"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-squash-multiple-success"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Squash destination"
Show
Sleep 2s
Caption "Undo restores the prior repository content; verify the files before continuing."
Type "au"
Wait+Screen "Undid operation"
Sleep 1s
Screenshot target/manual/manual-squash-multiple-recovered.png
State target/manual/manual-squash-multiple-recovered.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-squash-multiple-undone"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-squash-multiple-undone"
Type "exit"
Enter
