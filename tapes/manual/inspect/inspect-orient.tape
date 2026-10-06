# Manual scenario: inspect-orient
Output target/manual/inspect-orient.mp4
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1400
Set Height 840
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 70ms
Set WaitTimeout 30s

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" inspection"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show

Caption "Start with the working copy: the graph shows where the cache experiment belongs."
Wait+Screen "Experiment with forecast caching"
Sleep 2500ms
Screenshot target/manual/inspect-orient-log.png
State target/manual/inspect-orient-log.json

Caption "j and k move between changes; Space marks a revision without changing the repository."
Type "j"
Sleep 2s
Space
Wait+Screen "mark 1/1"
Sleep 2s
Type "c"
Type "k"
Caption "Enter opens the selected change: identity, description, then its patch."
Enter

Caption "Check the change identity before reading its patch. H or L returns to the log."
Wait+Screen "jk jj show"
Sleep 2500ms
Screenshot target/manual/inspect-orient-show.png
State target/manual/inspect-orient-show.json

Type "/"
Type "cache"
Enter
Wait+Screen "cache"
Sleep 2s
Type "n"
Sleep 2s
Type "L"
Wait+Screen "Experiment with forecast caching"
Caption "Status always describes the working copy, even when another log row is selected."
Type "j"
Sleep 2s
Type "s"

Caption "Status lists the changed paths and the working-copy and parent revisions."
Wait+Screen "Working copy changes:"
Sleep 2500ms
Screenshot target/manual/inspect-orient-status.png
State target/manual/inspect-orient-status.json

Type "L"
Wait+Screen "Experiment with forecast caching"
Type "L"
Wait+Screen "jk jj log"
Sleep 2s
Type "H"
Wait+Screen "Experiment with forecast caching"
Type "r"

Caption "Return, refresh, and continue from the same repository context."
Wait+Screen "Experiment with forecast caching"
Sleep 2500ms
Screenshot target/manual/inspect-orient-returned.png
State target/manual/inspect-orient-returned.json

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-orient"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
