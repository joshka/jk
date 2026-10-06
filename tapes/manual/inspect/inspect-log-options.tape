# Manual scenario: inspect-log-options
Output target/manual/inspect-log-options.mp4
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
Type@0ms "clear; \"$bin\" -R \"$repo\" log -n 8"
Enter
Wait+Screen "Experiment with forecast caching"
Show

Caption "V opens display options without changing the history."
Type "V"
Wait+Screen "Template"
Sleep 2s
Enter

Caption "Choose a jj template to balance graph density against description detail."
Wait+Screen "Log template"
Sleep 2500ms
Screenshot target/manual/inspect-log-options-selector.png
State target/manual/inspect-log-options-selector.json

Caption "Escape leaves the configured template in place."
Type "jj"
Sleep 2s
Escape
Wait+Screen "Experiment with forecast caching"
Sleep 2s
Caption "Comfortable separates revisions with space while retaining the description."
Type "V"
Enter
Wait+Screen "Log template"
Type "j"
Enter
Wait+Screen "builtin_log_comfortable"
Sleep 2s
Caption "Compact brings adjacent revisions closer together."
Type "V"
Enter
Type "j"
Enter
Wait+Screen "builtin_log_compact"
Sleep 2s
Type "V"
Enter
Type "j"
Enter

Caption "Full-description compact view keeps message detail close to its graph row."
Wait+Screen "builtin_log_compact_full_description"
Sleep 2500ms
Screenshot target/manual/inspect-log-options-full-description.png
State target/manual/inspect-log-options-full-description.json

Caption "Detailed exposes the metadata for each revision."
Type "V"
Enter
Type "j"
Enter
Wait+Screen "builtin_log_detailed"
Sleep 2s
Type "V"
Enter
Type "j"
Enter

Caption "One-line view fits more changes when you need to compare the graph."
Wait+Screen "builtin_log_oneline"
Sleep 2500ms
Screenshot target/manual/inspect-log-options-oneline.png
State target/manual/inspect-log-options-oneline.json

Caption "Redacted replaces identifying details while preserving the graph structure."
Type "V"
Enter
Type "j"
Enter
Wait+Screen "builtin_log_redacted"
Sleep 2s
Caption "Select configured again to restore the repository's normal jj rendering."
Type "V"
Enter
Type "j"
Enter

Caption "Changing templates affects presentation. The source changes and working copy stay intact."
Wait+Screen "Experiment with forecast caching"
Sleep 2500ms
Screenshot target/manual/inspect-log-options-configured.png
State target/manual/inspect-log-options-configured.json

Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "source \"$source_repo/scripts/manual-assert.sh\" inspection inspect-log-options"
Enter
Wait+Screen "JK_ASSERTIONS_PASSED"
Type "exit"
Enter
