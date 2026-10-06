Output target/manual/manual-restore.mp4
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" restore"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Restore working content"
Show
Sleep 2s
Caption "Select the old content. Restore replaces every path in the working copy."
Type "j"
Type "ar"
Wait+Screen "Affected content: all paths"
Sleep 1s
Screenshot target/manual/manual-restore-preview.png
State target/manual/manual-restore-preview.json
Sleep 2s
Escape
Wait+Screen "Restore base"
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-restore-cancelled"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-restore-cancelled"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Restore working content"
Show
Sleep 2s
Type "j"
Type "ar"
Wait+Screen "Affected content: all paths"
Sleep 2s
Enter
Wait+Screen "Restored all paths into working copy"
Sleep 1s
Screenshot target/manual/manual-restore-success.png
State target/manual/manual-restore-success.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-restore-success"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-restore-success"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Restore working content"
Show
Sleep 2s
Caption "Undo restores the prior repository content; verify the files before continuing."
Type "au"
Wait+Screen "Undid operation"
Sleep 1s
Screenshot target/manual/manual-restore-recovered.png
State target/manual/manual-restore-recovered.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-restore-undone"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-restore-undone"
Type "exit"
Enter
