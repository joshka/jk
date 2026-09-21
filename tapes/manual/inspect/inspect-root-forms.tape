# Manual scenario: inspect-root-forms
Output target/manual/inspect-root-forms.mp4
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
Type@0ms "jk() { \"$bin\" -R \"$repo\" \"$@\"; }; clear; \"$bin\" -R \"$repo\" log -n 3 -T builtin_log_oneline"
Enter
Wait+Screen "Experiment with forecast caching"
Show

Caption "Use log -n and -T to choose a bounded, one-line starting view."
Wait+Screen "builtin_log_oneline"
Sleep 2500ms
Screenshot target/manual/inspect-root-forms-custom-log.png
State target/manual/inspect-root-forms-custom-log.json

Caption "The show entry point accepts more than one revision."
Type "q"
Sleep 300ms
Ctrl+l
Type "jk show @ @-"
Enter

Caption "Show both the working-copy change and its parent in one inspection view."
Wait+Screen "jk jj show @ @-"
Sleep 2500ms
Screenshot target/manual/inspect-root-forms-multi-show.png
State target/manual/inspect-root-forms-multi-show.json

Caption "Page through the selected revisions, then open status with explicit filesets."
Space
Sleep 2s
Type "q"
Sleep 300ms
Ctrl+l
Type "jk status README.md config.json"
Enter

Caption "Filesets limit the listed paths; status still identifies the working copy and parent."
Wait+Screen "jk jj status README.md config.json"
Sleep 2500ms
Screenshot target/manual/inspect-root-forms-fileset-status.png
State target/manual/inspect-root-forms-fileset-status.json

Type "q"
Sleep 300ms
Caption "Omitting status filesets lists every changed path."
Ctrl+l
Type "jk status"
Enter

Caption "The unfiltered status also includes the cache implementation."
Wait+Screen "src/cache.js"
Sleep 2500ms
Screenshot target/manual/inspect-root-forms-full-status.png
State target/manual/inspect-root-forms-full-status.json

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-root-forms"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
