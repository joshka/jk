Output target/manual/manual-run-options-ignore.mp4
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" run-options"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Rebase source"
Show
Sleep 2s
Caption "Rebase at the latest operation while leaving the working-copy files on disk unchanged."
Type "R"
Wait+Screen "Choose rebase destination"
Type "/Rebase destination"
Enter
Wait+Screen "Source role:"
Type "o"
Wait+Screen "Run options"
Enter
Sleep 1s
Screenshot target/manual/manual-run-options-ignore-operation.png
State target/manual/manual-run-options-ignore-operation.json
Sleep 2s
Type "a"
Wait+Screen "--ignore-working-copy"
Caption "Ignore skips both snapshot and update for this command. Review the explicit flag before running."
Sleep 1s
Screenshot target/manual/manual-run-options-ignore-preview.png
State target/manual/manual-run-options-ignore-preview.json
Sleep 2s
Enter
Wait+Screen "Completed without updating working-copy files"
Sleep 1s
Screenshot target/manual/manual-run-options-ignore-completed.png
State target/manual/manual-run-options-ignore-completed.json
Sleep 2s
Type "C"
Wait+Screen "Command History"
Caption "Select the rebase record to inspect the command that left the working copy unchanged."
Type "jj"
Enter
Wait+Screen "Source: log rebase"
Wait+Screen "Status: success"
Caption "Command History retains the successful command and its exact execution options."
Sleep 1s
Screenshot target/manual/manual-run-options-ignore-operation-result.png
State target/manual/manual-run-options-ignore-operation-result.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-run-options-ignore"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-run-options-ignore"
Type "exit"
Enter
