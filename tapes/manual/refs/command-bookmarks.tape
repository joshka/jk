Output target/manual/command-bookmarks.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" refs-tracking"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Prepare remote workflow"
Type "B"
Wait+Screen "jj bookmark list"
Wait+Screen "incoming"
Show
Caption "The fetched incoming bookmark is remote-only. Track it through command mode."
Sleep 3s
Type ":bookmark track exact:incoming --remote exact:fixture"
Sleep 2s
Enter
Wait+Screen "Status: success"
Wait+Screen "Started tracking"
Sleep 3s
Backspace
Wait+Screen "jj bookmark list"
Caption "Refresh the bookmark view to see the new local tracking bookmark."
Type "r"
Wait+Screen "incoming"
Sleep 3s
Screenshot target/manual/command-bookmarks-tracked.png
State target/manual/command-bookmarks-tracked.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-bookmarks-tracked"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-bookmarks-tracked"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Prepare remote workflow"
Type "B"
Wait+Screen "jj bookmark list"
Show

Caption "Stop tracking that remote. The local incoming bookmark remains."
Type ":bookmark untrack exact:incoming --remote exact:fixture"
Sleep 2s
Enter
Wait+Screen "Status: success"
Sleep 3s
Backspace
Wait+Screen "jj bookmark list"
Type "r"
Wait+Screen "incoming"
Sleep 3s
Screenshot target/manual/command-bookmarks-untracked.png
State target/manual/command-bookmarks-untracked.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-bookmarks-untracked"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-bookmarks-untracked"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Prepare remote workflow"
Type "B"
Wait+Screen "jj bookmark list"
Show

Caption "Rename the local bookmark without changing its target or the remote's name."
Type ":bookmark rename incoming local-review"
Sleep 2s
Enter
Wait+Screen "Status: success"
Sleep 3s
Backspace
Wait+Screen "jj bookmark list"
Type "r"
Wait+Screen "local-review"
Sleep 3s
Screenshot target/manual/command-bookmarks-renamed.png
State target/manual/command-bookmarks-renamed.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-bookmarks-renamed"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-bookmarks-renamed"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Prepare remote workflow"
Type "B"
Wait+Screen "jj bookmark list"
Show

Caption "Forget the local name without queuing a deletion to push. The remote bookmark survives."
Type ":bookmark forget exact:local-review"
Sleep 2s
Enter
Wait+Screen "Status: success"
Wait+Screen "Forgot 1 local bookmarks"
Sleep 3s
Backspace
Wait+Screen "jj bookmark list"
Type "r"
Wait+Screen "incoming"
Sleep 3s
Screenshot target/manual/command-bookmarks-forgotten.png
State target/manual/command-bookmarks-forgotten.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-bookmarks-forgotten"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-bookmarks-forgotten"
