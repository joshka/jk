# Manual scenario: inspect-diff-formats
Output target/manual/inspect-diff-formats.mp4
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
Type@0ms "clear; \"$bin\" -R \"$repo\" diff --summary"
Enter
Wait+Screen "jk jj diff -r @ --summary"
Show

Caption "Summary answers which files changed and whether each was added, modified, or removed."
Wait+Screen "src/cache.js"
Sleep 2500ms
Screenshot target/manual/inspect-diff-formats-summary.png
State target/manual/inspect-diff-formats-summary.json

Caption "V changes the format of this same comparison."
Type "V"
Wait+Screen "summary"
Sleep 2s
Type "j"
Enter

Caption "Stat shows the size of the changes before you read every line."
Wait+Screen "--stat"
Sleep 2500ms
Screenshot target/manual/inspect-diff-formats-stat.png
State target/manual/inspect-diff-formats-stat.json

Type "V"
Type "j"
Enter

Caption "Types identifies file kinds on each side of the comparison."
Wait+Screen "--types"
Sleep 2500ms
Screenshot target/manual/inspect-diff-formats-types.png
State target/manual/inspect-diff-formats-types.json

Type "V"
Type "j"
Enter

Caption "Name-only gives a compact path list for deciding which files need review."
Wait+Screen "--name-only"
Sleep 2500ms
Screenshot target/manual/inspect-diff-formats-names.png
State target/manual/inspect-diff-formats-names.json

Caption "Escape from the format selector preserves the current name-only view."
Type "V"
Wait+Screen "color words"
Type "j"
Sleep 2s
Escape
Wait+Screen "--name-only"
Sleep 2s

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-diff-formats"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
