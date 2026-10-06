Output target/manual/refs-remotes.mp4

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
Caption "Inspect the configured remote before bringing its bookmarks into this repository."
Type "B"
Wait+Screen "jj bookmark list"
Sleep 2s
Type "F"
Wait+Screen "Fetch from remote"
Wait+Screen "fixture"
Sleep 3s
Screenshot target/manual/refs-remotes-choose.png
State target/manual/refs-remotes-choose.json
Enter
Wait+Screen "jj git fetch"
Wait+Screen "exact:"
Sleep 3s
Screenshot target/manual/refs-remotes-fetch-preview.png
State target/manual/refs-remotes-fetch-preview.json
Caption "Cancel the preview without contacting the remote, then reopen and confirm."
Escape
Wait+Screen "jj bookmark list"
Sleep 2s
Type "F"
Wait+Screen "Fetch from remote"
Enter
Wait+Screen "jj git fetch"
Sleep 2s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/refs-remotes-fetched.png
State target/manual/refs-remotes-fetched.json
Backspace
Wait+Screen "jj bookmark list"
Wait+Screen "incoming"
Sleep 2s

Caption "Select demo and preview a push to one explicit remote. This is a dry-run."
Home
Type "P"
Wait+Screen "Push dry-run: demo"
Sleep 2s
Enter
Wait+Screen "jj git push"
Wait+Screen "--dry-run"
Sleep 3s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/refs-remotes-push-result.png
State target/manual/refs-remotes-push-result.json
Caption "The proposed publication is retained for review. The remote refs were not changed."
Sleep 3s
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" refs-remotes"
Enter
Wait+Screen "MANUAL_ASSERT_OK refs-remotes"
