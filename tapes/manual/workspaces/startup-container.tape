Output target/manual/startup-container.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" container"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "cd \"$container_repo\"; jk() { \"$bin\" \"$@\"; }; clear"
Enter
Wait+Line
Show
Caption "Start jk from a metadata container, then check which working copy it selected."
Type "jk"
Enter
Wait+Screen "Container workspace review"
Sleep 2s
Type "s"
Wait+Screen "jj status"
Wait+Screen "No working copy"
Sleep 3s
Screenshot target/manual/startup-container-missing.png
State target/manual/startup-container-missing.json

Caption "Automatic selection fails here. Quit and explicitly select the default child workspace."
Sleep 3s
Type "q"
Sleep 300ms
Hide
Type@0ms "clear"
Enter
Wait+Line
Show
Type "jk -R default"
Enter
Wait+Screen "Container workspace review"
Sleep 3s
Screenshot target/manual/startup-container-recovered-log.png
State target/manual/startup-container-recovered-log.json

Caption "Status now shows the default child's working copy and its parent."
Type "s"
Wait+Screen "jj status"
Wait+Screen "Working copy  (@)"
Sleep 3s
Screenshot target/manual/startup-container-status.png
State target/manual/startup-container-status.json
Backspace
Wait+Screen "H home"
Type "W"
Wait+Screen "jj workspace list"
Wait+Screen "default"
Caption "The workspace list identifies default as the active repository scope."
Sleep 3s
Screenshot target/manual/startup-container-workspace.png
State target/manual/startup-container-workspace.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" startup-container"
Enter
Wait+Screen "MANUAL_ASSERT_OK startup-container"
