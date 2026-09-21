Output target/manual/manual-rebase-source-before.mp4
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
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" rebase"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "jj -R \"$repo\" config set --repo revsets.log 'description(substring:\"Rebase source\") | description(substring:\"Rebase destination\") | description(substring:\"Rebase base\")'; clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Rebase source"
Show
Sleep 2s
Caption "Move source scope before the destination; review the affected descendants."
Type "R"
Wait+Screen "Choose rebase destination"
Type "sB"
Type "/Rebase destination"
Sleep 1s
Screenshot target/manual/manual-rebase-source-before-selector.png
State target/manual/manual-rebase-source-before-selector.json
Sleep 2s
Enter
Wait+Screen "Source role:"
Caption "Read the exact source, destination, scope, and command before changing history."
Sleep 1s
Screenshot target/manual/manual-rebase-source-before-preview.png
State target/manual/manual-rebase-source-before-preview.json
Sleep 2s
Enter
Wait+Screen "Rebased revisions"
Caption "The refreshed graph and Command History confirm the operation succeeded."
Sleep 1s
Screenshot target/manual/manual-rebase-source-before-success.png
State target/manual/manual-rebase-source-before-success.json
Sleep 2s
Type "C"
Wait+Screen "Command History"
Sleep 1s
Screenshot target/manual/manual-rebase-source-before-history.png
State target/manual/manual-rebase-source-before-history.json
Sleep 2s
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" manual-rebase-source-before"
Enter
Wait+Screen "MANUAL_ASSERT_OK manual-rebase-source-before"
Type "exit"
Enter
