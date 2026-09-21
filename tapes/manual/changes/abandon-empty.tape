Output target/manual/abandon-empty.mp4

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
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Empty checkpoint"
Show
Sleep 3s
Caption "An empty change has no file changes. Abandon runs immediately."
Type "a"
Wait+Screen "Actions"
Sleep 3s
Screenshot target/manual/abandon-empty-action.png
State target/manual/abandon-empty-action.json
Type "a"
Wait+Screen "Abandoned revision"
Sleep 3s
Caption "jj creates a replacement working copy. The parent and its files remain."
Sleep 3s
Screenshot target/manual/abandon-empty-replacement.png
State target/manual/abandon-empty-replacement.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" abandon-empty-replaced"
Enter
Wait+Screen "MANUAL_ASSERT_OK abandon-empty-replaced"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 2s
Caption "Undo restores the original empty change and its description."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Wait+Screen "Empty checkpoint"
Sleep 3s
Screenshot target/manual/abandon-empty-undone.png
State target/manual/abandon-empty-undone.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" abandon-empty-undone"
Enter
Wait+Screen "MANUAL_ASSERT_OK abandon-empty-undone"
