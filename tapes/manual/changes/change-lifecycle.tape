Output target/manual/change-lifecycle.mp4

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
Caption "The cursor names the change to describe. Escape discards an unsaved message."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "m"
Wait+Screen "Describe revision"
Sleep 3s
Ctrl+u
Type "Discard this draft"
Sleep 2s
Screenshot target/manual/change-lifecycle-cancel.png
State target/manual/change-lifecycle-cancel.json
Escape
Wait+Screen "Experiment with forecast caching"
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" change-lifecycle-cancelled"
Enter
Wait+Screen "MANUAL_ASSERT_OK change-lifecycle-cancelled"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 2s
Caption "Enter saves. Ctrl-j inserts a newline inside the description."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "m"
Wait+Screen "Describe revision"
Sleep 2s
Ctrl+u
Type "Tune forecast cache lifetime"
Ctrl+j
Ctrl+j
Type "Keep cached forecasts fresh."
Sleep 3s
Screenshot target/manual/change-lifecycle-multiline.png
State target/manual/change-lifecycle-multiline.json
Enter
Wait+Screen "Described revision"
Sleep 3s
Caption "New immediately creates and selects an empty child of the cursor change."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "n"
Wait+Screen "Created new change"
Sleep 3s
Screenshot target/manual/change-lifecycle-new.png
State target/manual/change-lifecycle-new.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" change-lifecycle-new"
Enter
Wait+Screen "MANUAL_ASSERT_OK change-lifecycle-new"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Tune forecast cache lifetime"
Show
Sleep 2s
Caption "Move to the earlier change, then Edit makes it the working copy."
Type "j"
Sleep 2s
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "e"
Wait+Screen "Edited revision"
Sleep 3s
Caption "Undo restores the empty child. Redo returns to the earlier change."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Sleep 3s
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "U"
Wait+Screen "Redid operation"
Sleep 3s
Screenshot target/manual/change-lifecycle-recovered.png
State target/manual/change-lifecycle-recovered.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" change-lifecycle-edited"
Enter
Wait+Screen "MANUAL_ASSERT_OK change-lifecycle-edited"
