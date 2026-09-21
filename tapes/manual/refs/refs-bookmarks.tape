Output target/manual/refs-bookmarks.mp4

Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1280
Set Height 900
Set Padding 0
Set Margin 0
Set WindowBar ""
Set BorderRadius 0
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 80ms
Set WaitTimeout 60s

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" refs"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Prepare remote workflow"
Show
Caption "Name a change for review. The bookmark name can contain a literal asterisk."
Sleep 2s
Type "B"
Wait+Screen "jj bookmark list"
Wait+Screen "review-keep"
Sleep 2s
Type "c"
Wait+Screen "Bookmark mutation"
Type "review-*"
Tab
Type "-"
Sleep 2s
Enter
Wait+Screen "jj bookmark create"
Wait+Screen "review-*"
Sleep 3s
Screenshot target/manual/refs-bookmarks-create.png
State target/manual/refs-bookmarks-create.json
Enter
Wait+Screen "jj bookmark list"
Wait+Screen "review-*"
Sleep 2s
Home
Type "j"
Enter
Wait+Screen "jj show"
Sleep 3s
Backspace
Wait+Screen "jj bookmark list"

Caption "Move that one bookmark forward to the working change. Review the exact-name operand."
Type "m"
Wait+Screen "Bookmark mutation"
Sleep 2s
Enter
Wait+Screen "jj bookmark move"
Wait+Screen "exact:"
Sleep 3s
Screenshot target/manual/refs-bookmarks-move.png
State target/manual/refs-bookmarks-move.json
Enter
Wait+Screen "jj bookmark list"
Sleep 2s

Caption "Cancel deletion once. The bookmark and its target remain unchanged."
Type "x"
Wait+Screen "jj bookmark delete"
Wait+Screen "exact:"
Sleep 3s
Screenshot target/manual/refs-bookmarks-delete.png
State target/manual/refs-bookmarks-delete.json
Escape
Wait+Screen "jj bookmark list"
Sleep 2s
Caption "Confirm deletion. review-keep survives because the asterisk is matched literally."
Type "x"
Wait+Screen "jj bookmark delete"
Sleep 2s
Enter
Wait+Screen "Selected bookmark disappeared"
Wait+Screen "review-keep"
Sleep 3s

Caption "Recover the name by creating it again at the same working change."
Type "c"
Wait+Screen "Bookmark mutation"
Type "review-*"
Enter
Wait+Screen "jj bookmark create"
Sleep 2s
Enter
Wait+Screen "jj bookmark list"
Wait+Screen "review-*"
Sleep 3s
Screenshot target/manual/refs-bookmarks-recovered.png
State target/manual/refs-bookmarks-recovered.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" refs-bookmarks"
Enter
Wait+Screen "MANUAL_ASSERT_OK refs-bookmarks"
