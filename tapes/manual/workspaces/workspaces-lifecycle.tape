Output target/manual/workspaces-lifecycle.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" workspaces"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" workspaces"
Enter
Wait+Screen "jj workspace list"
Wait+Screen "scratch"
Show
Caption "Keep scratch: inspect the forget warning, then cancel."
Sleep 2s
Type "j"
Type "a"
Wait+Screen "Workspace lifecycle"
Sleep 2s
Type "f"
Wait+Screen "Files are not deleted"
Sleep 3s
Screenshot target/manual/workspaces-lifecycle-cancel.png
State target/manual/workspaces-lifecycle-cancel.json
Type "n"
Wait+Screen "jj workspace list"
Sleep 2s

Caption "Create a separate working directory. Its final path component becomes its workspace name."
Type "a"
Wait+Screen "Workspace lifecycle"
Type "a"
Wait+Screen "Destination path"
Type "../manual-added"
Sleep 2s
Enter
Wait+Screen "Exact command"
Wait+Screen "manual-added"
Sleep 3s
Screenshot target/manual/workspaces-lifecycle-add.png
State target/manual/workspaces-lifecycle-add.json
Enter
Wait+Screen "Add workspace succeeded"
Wait+Screen "manual-added"
Sleep 3s

Caption "Rename the selected workspace metadata. Its directory stays at manual-added."
Type "a"
Wait+Screen "Workspace lifecycle"
Type "r"
Wait+Screen "New workspace name"
Type "manual-renamed"
Enter
Wait+Screen "Exact command"
Wait+Screen "workspace rename"
Sleep 3s
Enter
Wait+Screen "Rename workspace succeeded"
Wait+Screen "manual-renamed"
Sleep 2s

Caption "A name collision fails without replacing either workspace."
Type "a"
Wait+Screen "Workspace lifecycle"
Type "r"
Wait+Screen "New workspace name"
Type "default"
Enter
Wait+Screen "workspace rename"
Sleep 2s
Enter
Wait+Screen "Rename workspace failed"
Sleep 3s
Screenshot target/manual/workspaces-lifecycle-collision.png
State target/manual/workspaces-lifecycle-collision.json

Caption "Command History opens on the last successful operation. Home selects the newest failed command."
Type "C"
Wait+Screen "Command History"
Home
Sleep 2s
Enter
Wait+Screen "Status: exit 1"
Sleep 3s
Backspace
Wait+Screen "Command History"
Backspace
Wait+Screen "jj workspace list"

Caption "Recover by reopening Rename and choosing an unused name."
Type "a"
Wait+Screen "Workspace lifecycle"
Type "r"
Wait+Screen "New workspace name"
Type "manual-ready"
Enter
Wait+Screen "workspace rename"
Sleep 2s
Enter
Wait+Screen "Rename workspace succeeded"
Wait+Screen "manual-ready"
Sleep 3s

Caption "Forget the new workspace metadata. Its directory and checked-out files remain on disk."
Type "a"
Wait+Screen "Workspace lifecycle"
Type "f"
Wait+Screen "Files are not deleted"
Wait+Screen "manual-ready"
Sleep 3s
Type "y"
Wait+Screen "Forget workspace metadata succeeded"
Wait+Screen "scratch"
Sleep 3s
Screenshot target/manual/workspaces-lifecycle-finished.png
State target/manual/workspaces-lifecycle-finished.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" workspaces-lifecycle"
Enter
Wait+Screen "MANUAL_ASSERT_OK workspaces-lifecycle"
