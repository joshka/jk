Output target/manual/workspaces-stale.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" workspaces-stale"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" workspaces"
Enter
Wait+Screen "jj workspace list"
Wait+Screen "scratch"
Show
Caption "Another workspace removed scratch's old README. Synchronize scratch's working directory."
Sleep 3s
Type "j"
Type "u"
Wait+Screen "Update stale workspace?"
Wait+Screen "scratch"
Sleep 3s
Screenshot target/manual/workspaces-stale-preview.png
State target/manual/workspaces-stale-preview.json
Caption "Cancel first: the preview has not updated the selected working copy."
Type "n"
Wait+Screen "jj workspace list"
Sleep 2s
Caption "Confirm the update to synchronize scratch while retaining its local work."
Type "u"
Wait+Screen "Update stale workspace?"
Sleep 2s
Enter
Wait+Screen "updated scratch via"
Caption "Scratch is synchronized; inspect the changed files next."
Sleep 3s
Screenshot target/manual/workspaces-stale-updated.png
State target/manual/workspaces-stale-updated.json

Caption "Verify the result: README was removed, and the local KEEP-ME file survives."
Enter
Wait+Screen "jj -R"
Wait+Screen "KEEP-ME.txt"
Wait+Screen "D ../scratch/README.md"
Wait+Screen "Refresh scratch description"
Sleep 3s
Screenshot target/manual/workspaces-stale-status.png
State target/manual/workspaces-stale-status.json
Backspace
Wait+Screen "jj workspace list"

Caption "Running update-stale again reports the already-current state."
Type "a"
Wait+Screen "Workspace lifecycle"
Type "u"
Wait+Screen "Update stale workspace?"
Sleep 2s
Enter
Wait+Screen "updated scratch via"
Wait+Screen "not stale"
Sleep 3s
Screenshot target/manual/workspaces-stale-current.png
State target/manual/workspaces-stale-current.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" workspaces-stale"
Enter
Wait+Screen "MANUAL_ASSERT_OK workspaces-stale"
