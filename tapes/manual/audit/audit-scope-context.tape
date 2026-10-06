Output target/manual/audit-scope-context.mp4

Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1280
Set Height 900
Set Padding 0
Set Margin 0
Set WindowBar ""
Set BorderRadius 0
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 80ms
Set WaitTimeout 60s

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/manual-fixture.sh\" workspaces"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "jj -R \"$repo\" bookmark create audit-context; jj -R \"$scratch_workspace\" status; python3 \"$source_repo/scripts/manual-audit-assert.py\" prepare \"$manual_real_jj\" \"$repo\" \"$scratch_workspace\" \"$manual_artifacts\""
Enter
Wait+Screen "MANUAL_AUDIT_BASELINE_READY"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Workspace lifecycle demo"
Type "B"
Wait+Screen "jj bookmark list"
Wait+Screen "audit-context"
Show
Caption "UX audit: the bookmark action-menu key enters a mode whose overlay is currently invisible."
Sleep 3s
Screenshot target/manual/audit-scope-context-bookmarks.png
State target/manual/audit-scope-context-bookmarks.json
Type "a"
Wait+Screen "jj bookmark list"
Sleep 3s
Screenshot target/manual/audit-scope-context-hidden-help.png
State target/manual/audit-scope-context-hidden-help.json
Escape
Wait+Screen "jj bookmark list"
Caption "Escape restores direct bookmark keys. The ordinary help key still displays useful bookmark controls."
Type "?"
Wait+Screen "Bookmarks keys"
Sleep 3s
Escape
Wait+Screen "jj bookmark list"
Type "H"
Wait+Screen "Workspace lifecycle demo"
Type "W"
Wait+Screen "jj workspace list"
Wait+Screen "scratch"
Type "j"
Enter
Wait+Screen "KEEP-ME.txt"
Caption "We are inspecting scratch. The arbitrary command prompt does not name its execution workspace."
Sleep 3s
Type ":describe -m 'Scope audit changed startup workspace' @"
Sleep 3s
Screenshot target/manual/audit-scope-context-input.png
State target/manual/audit-scope-context-input.json
Enter
Wait+Screen "Status: success"
Caption "The command succeeded. Repository assertions show it changed default; scratch stayed unchanged."
Sleep 4s
Screenshot target/manual/audit-scope-context-result.png
State target/manual/audit-scope-context-result.json
Backspace
Wait+Screen "KEEP-ME.txt"
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "python3 \"$source_repo/scripts/manual-audit-assert.py\" scope \"$manual_real_jj\" \"$repo\" \"$scratch_workspace\" \"$manual_artifacts\""
Enter
Wait+Screen "MANUAL_AUDIT_SCOPE_CONFIRMED"
Type@0ms "clear; \"$bin\" -R \"$repo\" -n 8"
Enter
Wait+Screen "Scope audit changed startup workspace"
Show
Caption "Recover in the repository that actually changed: Undo restores its previous description."
Sleep 3s
Type "a"
Wait+Screen "Undo"
Type "u"
Wait+Screen "Undid operation"
Wait+Screen "Workspace lifecycle demo"
Sleep 3s
Screenshot target/manual/audit-scope-context-recovered.png
State target/manual/audit-scope-context-recovered.json
Caption ""
Hide
Type "q"
Sleep 300ms
Type@0ms "python3 \"$source_repo/scripts/manual-audit-assert.py\" recovered \"$manual_real_jj\" \"$repo\" \"$scratch_workspace\" \"$manual_artifacts\""
Enter
Wait+Screen "MANUAL_AUDIT_RECOVERED_CONFIRMED"
