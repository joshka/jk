Output target/manual/command-filesets.mp4

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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" command-changes"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 3s
Caption "A fileset and message let split run without opening a foreground editor."
Type ":split -m 'Separate cache policy' config.json"
Sleep 3s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/command-filesets-split-output.png
State target/manual/command-filesets-split-output.json
Caption "Refresh the log: the selected path and remaining work are now separate changes."
Backspace
Wait+Screen "jk jj"
Type "r"
Wait+Screen "Separate cache policy"
Sleep 3s
Screenshot target/manual/command-filesets-split-graph.png
State target/manual/command-filesets-split-graph.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-filesets-split"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-filesets-split"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Separate cache policy"
Show
Sleep 2s
Caption "Undo recombines both paths before trying a path-specific restore."
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "u"
Wait+Screen "Undid operation"
Sleep 3s
Caption "Restore config.json from the parent. The cache implementation stays in the working copy."
Type ":restore --from @- config.json"
Sleep 3s
Enter
Wait+Screen "Status: success"
Sleep 3s
Screenshot target/manual/command-filesets-restore-output.png
State target/manual/command-filesets-restore-output.json
Backspace
Wait+Screen "jk jj"
Type "r"
Wait+Screen "Experiment with forecast caching"
Sleep 2s
Caption "The remaining diff contains the new cache implementation."
Type "d"
Wait+Screen "src/cache.js"
Sleep 3s
Screenshot target/manual/command-filesets-retained.png
State target/manual/command-filesets-retained.json
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" command-filesets-restored"
Enter
Wait+Screen "MANUAL_ASSERT_OK command-filesets-restored"
