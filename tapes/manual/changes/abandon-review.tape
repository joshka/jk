Output target/manual/abandon-review.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" abandon"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Document cache metrics"
Show
Sleep 3s
Caption "Select the cache change. Its descendants are separate changes."
Type "j"
Sleep 800ms
Type "j"
Sleep 3s
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "a"
Wait+Screen "Abandon this change?"
Sleep 3s
Screenshot target/manual/abandon-review-decision.png
State target/manual/abandon-review-decision.json
Caption "View diff shows the changes that will be removed."
Left
Enter
Wait+Screen "Changes to abandon"
Sleep 3s
Screenshot target/manual/abandon-review-diff.png
State target/manual/abandon-review-diff.json
Caption "Escape returns to the decision. Escape again cancels the whole action."
Escape
Wait+Screen "Abandon this change?"
Sleep 2s
Escape
Wait+Screen "Tune forecast caching"
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" abandon-review-cancelled"
Enter
Wait+Screen "MANUAL_ASSERT_OK abandon-review-cancelled"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Document cache metrics"
Show
Sleep 2s
Caption "Reopen the same change and confirm. jj rebases its descendants."
Type "j"
Sleep 800ms
Type "j"
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "a"
Wait+Screen "Abandon this change?"
Sleep 3s
Type "y"
Wait+Screen "Abandoned revision"
Sleep 3s
Screenshot target/manual/abandon-review-abandoned.png
State target/manual/abandon-review-abandoned.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" abandon-review-abandoned"
Enter
Wait+Screen "MANUAL_ASSERT_OK abandon-review-abandoned"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Document cache metrics"
Show
Sleep 2s
Caption "Undo restores the removed change, its files, and the original descendants."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Wait+Screen "Tune forecast caching"
Sleep 3s
Screenshot target/manual/abandon-review-restored.png
State target/manual/abandon-review-restored.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" abandon-review-undone"
Enter
Wait+Screen "MANUAL_ASSERT_OK abandon-review-undone"
