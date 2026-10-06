Output target/manual/manual-run-options-immutable.mp4
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" run-options-immutable"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Rebase source"
Show
Sleep 2s
Caption "Move revision scope onto the destination; review the affected descendants."
Type "R"
Wait+Screen "Choose rebase destination"
Type "ro"
Type "/Rebase destination"
Sleep 1s
Screenshot target/manual/manual-run-options-immutable-selector.png
State target/manual/manual-run-options-immutable-selector.json
Sleep 2s
Enter
Wait+Screen "Source role:"
Caption "Read the exact source, destination, scope, and command before changing history."
Sleep 1s
Screenshot target/manual/manual-run-options-immutable-preview.png
State target/manual/manual-run-options-immutable-preview.json
Sleep 2s
Escape
Wait+Screen "Rebase source"
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-run-options-immutable-cancelled"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-run-options-immutable-cancelled"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Rebase source"
Show
Sleep 2s
Type "R"
Wait+Screen "Choose rebase destination"
Type "/Rebase destination"
Enter
Wait+Screen "Source role:"
Sleep 2s
Enter
Wait+Screen "jj rebase failed"
Type "C"
Wait+Screen "Command History"
Enter
Wait+Screen "immutable"
Caption "Command history retains the full explanation: this source is immutable."
Sleep 1s
Screenshot target/manual/manual-run-options-immutable-rejected.png
State target/manual/manual-run-options-immutable-rejected.json
Sleep 2s
Backspace
Wait+Screen "Command History"
Backspace
Wait+Screen "Rebase source"
Type "R"
Wait+Screen "Choose rebase destination"
Type "/Rebase destination"
Enter
Wait+Screen "Source role:"
Type "o"
Wait+Screen "Run options"
Type "jj"
Enter
Type "a"
Wait+Screen "--ignore-immutable"
Caption "Allowing immutable edits is a deliberate command-local override visible before confirmation."
Sleep 1s
Screenshot target/manual/manual-run-options-immutable-override.png
State target/manual/manual-run-options-immutable-override.json
Sleep 2s
Enter
Wait+Screen "Rebased revisions"
Sleep 1s
Screenshot target/manual/manual-run-options-immutable-success.png
State target/manual/manual-run-options-immutable-success.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-run-options-immutable-success"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-run-options-immutable-success"
Type "exit"
Enter
