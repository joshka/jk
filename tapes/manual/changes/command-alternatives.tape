Output target/manual/command-alternatives.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" command-changes"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 3s
Caption "Commit needs a message because command mode cannot open an editor. It acts on @."
Type ":commit -m 'Finalize forecast caching'"
Sleep 2s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/command-alternatives-commit.png
State target/manual/command-alternatives-commit.json
Caption "Return to the log and refresh to show the repository after a command-mode mutation."
Backspace
Wait+Screen "jk jj"
Sleep 2s
Type "r"
Wait+Screen "jk jj"
Sleep 3s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-alternatives-committed"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-alternatives-committed"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Finalize forecast caching"
Show
Sleep 2s
Caption "Undo the commit before comparing another operation."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Sleep 2s
Caption "Duplicate creates a new change identity with the same patch."
Type ":duplicate @"
Sleep 2s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/command-alternatives-duplicate.png
State target/manual/command-alternatives-duplicate.json
Caption "Return to the log and refresh to show the repository after a command-mode mutation."
Backspace
Wait+Screen "jk jj"
Sleep 2s
Type "r"
Wait+Screen "jk jj"
Sleep 3s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-alternatives-duplicated"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-alternatives-duplicated"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 2s
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Sleep 2s
Caption "Revert creates a reverse patch on the requested destination. It keeps the original."
Type ":revert -r @ -o @"
Sleep 2s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/command-alternatives-revert.png
State target/manual/command-alternatives-revert.json
Caption "Return to the log and refresh to show the repository after a command-mode mutation."
Backspace
Wait+Screen "jk jj"
Sleep 2s
Type "r"
Wait+Screen "jk jj"
Sleep 3s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-alternatives-reverted"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-alternatives-reverted"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 2s
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Sleep 2s
Caption "Absorb moves edits to the ancestors that last changed those lines. New files stay in @."
Type ":absorb"
Sleep 2s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/command-alternatives-absorb.png
State target/manual/command-alternatives-absorb.json
Caption "Return to the log and refresh to show the repository after a command-mode mutation."
Backspace
Wait+Screen "jk jj"
Sleep 2s
Type "r"
Wait+Screen "jk jj"
Sleep 3s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-alternatives-absorbed"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-alternatives-absorbed"
