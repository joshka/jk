Output target/manual/manual-command-restrictions.mp4
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
Set WaitTimeout 30s
Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" refs"
Enter
Wait+Screen "JK_MANUAL_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Review cache publication"
Show
Sleep 2s
Caption "Bookmark mutations require the native screen so their exact operands can be reviewed."
Type ":bookmark create review-topic"
Enter
Wait+Screen "bookmark changes are unavailable"
Sleep 1s
Screenshot target/manual/manual-command-restrictions-bookmark-blocked.png
State target/manual/manual-command-restrictions-bookmark-blocked.json
Sleep 2s
Escape
Type ":bookmark move demo --to @-"
Enter
Wait+Screen "bookmark changes are unavailable"
Sleep 2s
Escape
Type ":bookmark delete demo"
Enter
Wait+Screen "bookmark changes are unavailable"
Sleep 2s
Escape
Caption "Fetch and push are also routed through the bookmark screen, including push dry-run."
Type ":git fetch"
Enter
Wait+Screen "F action"
Sleep 2s
Escape
Type ":git push"
Enter
Wait+Screen "git push is unavailable"
Sleep 2s
Escape
Type ":git push --dry-run"
Enter
Wait+Screen "git push is unavailable"
Sleep 1s
Screenshot target/manual/manual-command-restrictions-push-blocked.png
State target/manual/manual-command-restrictions-push-blocked.json
Sleep 2s
Escape
Caption "B opens bookmarks. F selects a remote and shows the native fetch confirmation."
Type "B"
Wait+Screen "jj bookmark list"
Type "F"
Wait+Screen "Fetch from remote"
Enter
Wait+Screen "jj git fetch"
Sleep 1s
Screenshot target/manual/manual-command-restrictions-native-path.png
State target/manual/manual-command-restrictions-native-path.json
Sleep 2s
Escape
Escape
Hide
Caption ""
Type "q"
Wait+Line
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection manual-command-restrictions"
Enter
Wait+Screen "JK_MANUAL_ASSERTIONS_PASSED"
Type "exit"
Enter
