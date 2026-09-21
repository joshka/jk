Output target/manual/workspaces-inspect.mp4

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
Caption "Inspect another workspace from one session. Start by selecting scratch."
Sleep 3s
Type "j"
Enter
Wait+Screen "jj -R"
Wait+Screen "KEEP-ME.txt"
Sleep 3s
Screenshot target/manual/workspaces-inspect-status.png
State target/manual/workspaces-inspect-status.json
Backspace
Wait+Screen "jj workspace list"
Caption "The workspace diff uses scratch's working copy, including its new file."
Type "d"
Wait+Screen "jj -R"
Wait+Screen "KEEP-ME.txt"
Sleep 3s
Screenshot target/manual/workspaces-inspect-diff.png
State target/manual/workspaces-inspect-diff.json
Backspace
Wait+Screen "jj workspace list"

Caption "Review scratch's history. The workspace log does not open individual changes."
Type "l"
Wait+Screen "jj -R"
Wait+Screen "scratch"
Sleep 3s
Backspace
Wait+Screen "jj workspace list"
Caption "For change details, command mode needs an explicit workspace revision: scratch@."
Type ":show -r scratch@"
Sleep 2s
Enter
Wait+Screen "Status: success"
Wait+Screen "KEEP-ME.txt"
Sleep 3s
Screenshot target/manual/workspaces-inspect-show.png
State target/manual/workspaces-inspect-show.json
Backspace
Wait+Screen "jj workspace list"
Caption "Inspect that same change's evolution with an explicit scratch@ operand."
Type ":evolog -r scratch@"
Sleep 2s
Enter
Wait+Screen "Status: success"
Wait+Screen "scratch@"
Sleep 3s
Screenshot target/manual/workspaces-inspect-evolog.png
State target/manual/workspaces-inspect-evolog.json
Backspace
Wait+Screen "jj workspace list"
Type "r"
Wait+Screen "scratch"
Sleep 2s
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" workspaces-inspect"
Enter
Wait+Screen "MANUAL_ASSERT_OK workspaces-inspect"
