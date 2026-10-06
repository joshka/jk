# Manual scenario: inspect-diff-navigate
Output target/manual/inspect-diff-navigate.mp4
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
Type@0ms "clear; \"$bin\" -R \"$repo\" diff -r @ --color-words"
Enter
Wait+Screen "jk jj diff -r @ --color-words"
Show

Caption "Review the working-copy patch against its parent before making a source-control decision."
Wait+Screen "Modified regular file"
Sleep 2500ms
Screenshot target/manual/inspect-diff-navigate-patch.png
State target/manual/inspect-diff-navigate-patch.json

Caption "f lists paths so you can jump straight to the cache implementation."
Type "f"

Caption "Select src/cache.js from the file list; Escape would close it without moving."
Wait+Screen "Diff files"
Wait+Screen "3/3 src/cache.js"
Sleep 2500ms
Screenshot target/manual/inspect-diff-navigate-files.png
State target/manual/inspect-diff-navigate-files.json

Type "jj"
Enter
Wait+Screen "src/cache.js"
Sleep 2s
Caption "Search the patch; n and N move between matching text."
Type "/"
Type "cache"
Enter
Wait+Screen "cache"
Sleep 2s
Type "n"
Sleep 2s
Type "N"
Caption "Return to the first file, then use ] twice to select the cache implementation."
Type "g]]"
Wait+Screen "file 3/3"
Sleep 2s
Type "h"

Caption "Fold the selected file to retain its path and line counts; l restores its contents."
Wait+Screen "/(?s)Modified regular file src/cache[.]js:[^\\n]*\\n\\s*/cache/"
Sleep 2500ms
Screenshot target/manual/inspect-diff-navigate-folded-file.png
State target/manual/inspect-diff-navigate-folded-file.json

Type "l"
Wait+Screen "cacheLimit"
Sleep 2s
Type ">"
Sleep 2s
Type "<"
Type "["
Sleep 2s
Type "]"
Type "r"

Caption "Refresh rereads the same revision. Inspection and folding have changed no repository data."
Wait+Screen "jk jj diff -r @ --color-words"
Wait+Screen "cacheLimit"
Sleep 2500ms
Screenshot target/manual/inspect-diff-navigate-refreshed.png
State target/manual/inspect-diff-navigate-refreshed.json

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-diff-navigate"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
