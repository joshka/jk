Output target/manual/abandon-probe-failure.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" empty"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "mkdir -p \"$manual_root/fake-bin\"; ln -s \"$source_repo/scripts/manual-fixture-jj.sh\" \"$manual_root/fake-bin/jj\"; export JK_REAL_JJ=\"$manual_real_jj\" JK_MANUAL_FAULT=abandon-probe; export PATH=\"$manual_root/fake-bin:$PATH\""
Enter
Wait+Line
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Empty checkpoint"
Show
Sleep 3s
Caption "When the emptiness check fails, jk asks for a decision instead of abandoning immediately."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "a"
Wait+Screen "Couldn't check whether this change is empty."
Sleep 3s
Screenshot target/manual/abandon-probe-failure-warning.png
State target/manual/abandon-probe-failure-warning.json
Caption "Cancel preserves the selected change and its repository state."
Escape
Wait+Screen "Empty checkpoint"
Sleep 3s
Screenshot target/manual/abandon-probe-failure-cancelled.png
State target/manual/abandon-probe-failure-cancelled.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" abandon-probe-failure"
Enter
Wait+Screen "MANUAL_ASSERT_OK abandon-probe-failure"
