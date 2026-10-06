# Manual scenario: inspect-diff-compare
Output target/manual/inspect-diff-compare.mp4
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
Type@0ms "jk() { \"$bin\" -R \"$repo\" \"$@\"; }; clear; \"$bin\" -R \"$repo\" diff --from main --to candidate --git"
Enter
Wait+Screen "jk jj diff --from main --to candidate --git"
Show

Caption "Compare two trees: main is the starting content, candidate is the ending content."
Wait+Screen "src/cache.js"
Sleep 2500ms
Screenshot target/manual/inspect-diff-compare-range.png
State target/manual/inspect-diff-compare-range.json

Caption "The same tree comparison can summarize paths, line counts, or file kinds."
Type "V"
Wait+Screen "summary"
Type "kkkkk"
Enter
Wait+Screen "Modified regular file"
Sleep 2s
Type "V"
Type "j"
Enter
Wait+Screen "jj diff --from main --to candidate --summary"
Sleep 2s
Type "V"
Type "j"
Enter
Wait+Screen "jj diff --from main --to candidate --stat"
Sleep 2s
Type "V"
Type "j"
Enter
Wait+Screen "jj diff --from main --to candidate --types"
Sleep 2s
Type "V"
Type "j"
Enter
Wait+Screen "jj diff --from main --to candidate --name-only"
Sleep 2s
Type "V"
Type "j"
Enter
Wait+Screen "jj diff --from main --to candidate --git"
Sleep 2s

Caption "Color-words changes presentation while preserving the from/to operands."
Type "V"
Wait+Screen "color words"
Type "j"
Enter

Caption "Read inline changes with jj color-words output for the same tree comparison."
Wait+Screen "--color-words"
Sleep 2500ms
Screenshot target/manual/inspect-diff-compare-color-words.png
State target/manual/inspect-diff-compare-color-words.json

Type "q"
Sleep 300ms
Caption "A positional revision is an alias for -r: its patch is relative to its parent."
Ctrl+l
Type "jk diff @"
Enter

Caption "The configured patch format compares one revision with its parent, rather than two chosen trees."
Wait+Screen "jk jj diff -r @"
Sleep 2500ms
Screenshot target/manual/inspect-diff-compare-revision.png
State target/manual/inspect-diff-compare-revision.json

Caption "The explicit color-words flag also works as a root command."
Type "q"
Sleep 300ms
Ctrl+l
Type "jk diff -r @ --color-words"
Enter

Caption "Use an explicit format at startup when that is the comparison you intend to inspect."
Wait+Screen "--color-words"
Sleep 2500ms
Screenshot target/manual/inspect-diff-compare-root-color-words.png
State target/manual/inspect-diff-compare-root-color-words.json

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-diff-compare"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
