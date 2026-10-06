Output target/manual/new-parents.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" parents"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Document cache policy"
Show
Sleep 3s
Caption "Marks become parents in the order you choose them. The cursor is independent."
Space
Sleep 2s
Type "j"
Space
Sleep 3s
Screenshot target/manual/new-parents-ordered.png
State target/manual/new-parents-ordered.json
Caption "New uses both marked parents and moves the working copy to their empty merge."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "n"
Wait+Screen "Created new change"
Sleep 3s
Screenshot target/manual/new-parents-merged.png
State target/manual/new-parents-merged.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" new-parents-merged"
Enter
Wait+Screen "MANUAL_ASSERT_OK new-parents-merged"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Document cache policy"
Show
Sleep 2s
Caption "Undo removes the merge and restores the previous working copy."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Sleep 3s
Screenshot target/manual/new-parents-undone.png
State target/manual/new-parents-undone.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" new-parents-undone"
Enter
Wait+Screen "MANUAL_ASSERT_OK new-parents-undone"
