Output target/manual/manual-dialog-light.mp4
Require jj
Set Shell "bash"
Set Theme "Builtin Light"
Set FontSize 20
Set Width 1400
Set Height 900
Set Padding 0
Set Margin 20
Set MarginFill "#f1f5f9"
Set WindowBar Colorful
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 80ms
Set WaitTimeout 45s
Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" changes"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" --dialog-theme light -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 2s
Caption "Choose a dialog palette that matches the terminal background."
Type "a"
Wait+Screen "Actions"
Sleep 1s
Screenshot target/manual/manual-dialog-light-actions.png
State target/manual/manual-dialog-light-actions.json
Sleep 2s
Escape
Type "am"
Wait+Screen "Describe revision"
Sleep 1s
Screenshot target/manual/manual-dialog-light-description.png
State target/manual/manual-dialog-light-description.json
Sleep 2s
Escape
Wait+Screen "Experiment with forecast caching"
Type "?"
Wait+Screen "Open and inspect"
Sleep 1s
Screenshot target/manual/manual-dialog-light-help.png
State target/manual/manual-dialog-light-help.json
Sleep 2s
Escape
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-dialog-light"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-dialog-light"
Type "exit"
Enter
