Output target/manual/manual-cancel-refresh.mp4
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
Set WaitTimeout 45s
Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" changes"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "mkdir -p \"$manual_root/fake-bin\" \"$manual_root/refresh-state\"; ln -s \"$source_repo/scripts/betamax-cancellable-jj.sh\" \"$manual_root/fake-bin/jj\"; export JK_REAL_JJ=\"$manual_real_jj\" JK_FAKE_JJ_STATE=\"$manual_root/refresh-state\" PATH=\"$manual_root/fake-bin:$PATH\""
Enter
Type@0ms "clear; \"$bin\" -R \"$repo\"  -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show
Sleep 2s
Caption "A slow refresh keeps the last usable graph visible while navigation remains responsive."
Type "r"
Wait+Screen "Refreshing"
Sleep 1s
Screenshot target/manual/manual-cancel-refresh-loading.png
State target/manual/manual-cancel-refresh-loading.json
Sleep 2s
Type "j"
Sleep 1s
Caption "A second refresh supersedes the slow process and loads the current repository."
Type "r"
Wait+Screen "Refreshing"
Sleep 2s
Sleep 1s
Screenshot target/manual/manual-cancel-refresh-replaced.png
State target/manual/manual-cancel-refresh-replaced.json
Sleep 2s
Caption "A failed refresh preserves the graph and reports the failure."
Type "r"
Wait+Screen "Refresh failed:"
Sleep 1s
Screenshot target/manual/manual-cancel-refresh-failure.png
State target/manual/manual-cancel-refresh-failure.json
Sleep 2s
Type "C"
Wait+Screen "Command History"
Enter
Wait+Screen "fixture refresh failed"
Sleep 1s
Screenshot target/manual/manual-cancel-refresh-details.png
State target/manual/manual-cancel-refresh-details.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-cancel-refresh"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-cancel-refresh"
Type "exit"
Enter
