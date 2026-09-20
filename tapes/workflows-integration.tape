Output target/dogfood-artifacts/betamax/workflows-integration.gif

Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1400
Set Height 900
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" workflows"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Show

Caption "Rebase previews an exact destination, and Escape leaves the graph unchanged."
Hide
Type@0ms "cd \"$rebase_repo\" && clear"
Enter
Wait
Show
Type "\"$bin\" -R \"$rebase_repo\" -n 5"
Enter
Wait+Screen "jk"
State target/dogfood-artifacts/betamax/workflows-rebase-initial.json
Wait+Screen "Rebase source"
Sleep 700ms
Type " "
Type "j"
Type "R"
Wait+Screen "Choose rebase destination"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-selector.png
State target/dogfood-artifacts/betamax/workflows-rebase-selector.json
Type "/destination"
Enter
Wait+Screen "Source role:"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-preview.png
State target/dogfood-artifacts/betamax/workflows-rebase-preview.json
Escape
Wait+Screen "Rebase destination"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-cancelled.png
State target/dogfood-artifacts/betamax/workflows-rebase-cancelled.json

Hide
Type "q"
Wait
Type "test \"$(jj -R \"$rebase_repo\" log --no-graph -r @ -T 'parents.map(|x| x.description().first_line()).join(\",\")')\" = \"Rebase base\" && printf 'JK_%s\\n' REBASE_CANCELLED"
Enter
Wait+Screen "JK_REBASE_CANCELLED"

Caption "Confirming the same rebase refreshes the log and records its resulting operation."
Show
Hide
Type@0ms "cd \"$rebase_repo\" && clear"
Enter
Wait
Show
Type "\"$bin\" -R \"$rebase_repo\" -n 5"
Enter
Wait+Screen "Rebase source"
Type " "
Type "j"
Type "R"
Wait+Screen "Choose rebase destination"
Enter
Wait+Screen "Source role:"
Enter
Wait+Screen "u undo"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-success.png
State target/dogfood-artifacts/betamax/workflows-rebase-success.json
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-history.png
State target/dogfood-artifacts/betamax/workflows-rebase-history.json
Type "o"
Wait+Screen "Changed commits"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-operation.png
State target/dogfood-artifacts/betamax/workflows-rebase-operation.json
Hide
Type "q"
Wait
Type "test \"$(jj -R \"$rebase_repo\" log --no-graph -r @ -T 'parents.map(|x| x.description().first_line()).join(\",\")')\" = \"Rebase destination\" && printf 'JK_%s\\n' REBASE_SUCCESS"
Enter
Wait+Screen "JK_REBASE_SUCCESS"

Caption "Squash preserves marked sources and the destination cursor when its preview is cancelled."
Show
Hide
Type@0ms "cd \"$squash_repo\" && clear"
Enter
Wait
Show
Type "\"$bin\" -R \"$squash_repo\" -n 5"
Enter
Wait+Screen "Squash source"
Sleep 700ms
Type " "
Type "j"
Type "a"
Wait+Screen "Squash"
Type "s"
Wait+Screen "Source 1:"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-preview.png
State target/dogfood-artifacts/betamax/workflows-squash-preview.json
Escape
Wait+Screen "Squash destination"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-cancelled.png
State target/dogfood-artifacts/betamax/workflows-squash-cancelled.json

Caption "The confirmed squash has an auditable Command History entry and operation link."
Type "a"
Type "s"
Wait+Screen "Source 1:"
Enter
Wait+Screen "Squashed source changes"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-success.png
State target/dogfood-artifacts/betamax/workflows-squash-success.json
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-history.png
State target/dogfood-artifacts/betamax/workflows-squash-history.json
Type "o"
Wait+Screen "Changed commits"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-operation.png
State target/dogfood-artifacts/betamax/workflows-squash-operation.json
Hide
Type "q"
Wait
Type@0ms "test \"$(cat \"$squash_repo/source.txt\")\" = 'source contents' && printf 'JK_%s\\n' SQUASH_CONTENT_RETAINED"
Enter
Wait+Screen "JK_SQUASH_CONTENT_RETAINED"

Caption "Restore names the source, working-copy destination, and destructive all-path scope."
Show
Hide
Type@0ms "cd \"$restore_repo\" && clear"
Enter
Wait
Show
Type "\"$bin\" -R \"$restore_repo\" -n 3"
Enter
Wait+Screen "Restore working content"
Sleep 700ms
Type "j"
Type "a"
Wait+Screen "Actions"
Type "r"
Wait+Screen "Affected content: all paths"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-preview.png
State target/dogfood-artifacts/betamax/workflows-restore-preview.json
Escape
Wait+Screen "Restore base"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-cancelled.png
State target/dogfood-artifacts/betamax/workflows-restore-cancelled.json

Caption "Confirming restore refreshes the working copy and captures its recovery operation."
Type "a"
Type "r"
Wait+Screen "Affected content: all paths"
Enter
Wait+Screen "Restored all paths into working copy"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-success.png
State target/dogfood-artifacts/betamax/workflows-restore-success.json
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-history.png
State target/dogfood-artifacts/betamax/workflows-restore-history.json
Type "o"
Wait+Screen "Changed commits"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-operation.png
State target/dogfood-artifacts/betamax/workflows-restore-operation.json
Hide
Type "q"
Wait
Type@0ms "test \"$(cat \"$restore_repo/file.txt\")\" = 'base content' && printf 'JK_%s\\n' RESTORE_CONTENT_MATCHES"
Enter
Wait+Screen "JK_RESTORE_CONTENT_MATCHES"

Caption "Workspace lifecycle keeps add and forget together, with a cancel-safe confirmation."
Show
Hide
Type@0ms "cd \"$workspaces_repo\" && clear"
Enter
Wait
Show
Type "\"$bin\" -R \"$workspaces_repo\" workspaces"
Enter
Wait+Screen "scratch"
Sleep 700ms
Type "j"
Type "a"
Wait+Screen "Workspace lifecycle"
Type "f"
Wait+Screen "Files are not deleted"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-forget-preview.png
State target/dogfood-artifacts/betamax/workflows-workspace-forget-preview.json
Type "n"
Wait+Screen "scratch"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-cancelled.png
State target/dogfood-artifacts/betamax/workflows-workspace-cancelled.json

Caption "Adding a workspace confirms exact argv, refreshes the list, and records an operation."
Type "a"
Type "a"
Wait+Screen "Destination path"
Type "../one/demo-added"
Enter
Wait+Screen "Exact command"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-add-preview.png
State target/dogfood-artifacts/betamax/workflows-workspace-add-preview.json
Enter
Wait+Screen "demo-added"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-success.png
State target/dogfood-artifacts/betamax/workflows-workspace-success.json

Caption "A second path with the same workspace name fails safely and stays visible in history."
Type "a"
Type "a"
Wait+Screen "Destination path"
Type "../two/demo-added"
Enter
Wait+Screen "Exact command"
Enter
State target/dogfood-artifacts/betamax/workflows-workspace-failure-state.json
Wait+Screen "failed:"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-failure.png
State target/dogfood-artifacts/betamax/workflows-workspace-failure.json

Caption "Rename updates only the selected workspace after reviewing the exact command."
Type "a"
Type "r"
Wait+Screen "New workspace name"
Type "renamed-demo"
Enter
Wait+Screen "Exact command"
Enter
Wait+Screen "renamed-demo"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-rename.png
State target/dogfood-artifacts/betamax/workflows-workspace-rename.json

Caption "Confirming forget removes metadata but retains the workspace directory and files."
Type "a"
Type "f"
Wait+Screen "Files are not deleted"
Type "y"
Wait+Screen "default"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-forgotten.png
State target/dogfood-artifacts/betamax/workflows-workspace-forgotten.json
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-history.png
State target/dogfood-artifacts/betamax/workflows-workspace-history.json
Type "j"
Type "j"
Type "j"
Type "j"
Type "j"
Type "j"
Type "j"
Type "o"
Wait+Screen "Changed commits"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-operation.png
State target/dogfood-artifacts/betamax/workflows-workspace-operation.json
Caption ""
Hide
Type "q"
Wait
Type "test -f \"$fixture_root/workspaces/one/demo-added/README.md\" && cmp \"$workspaces_repo/README.md\" \"$fixture_root/workspaces/one/demo-added/README.md\" && printf 'JK_%s\\n' WORKSPACE_FILES_RETAINED"
Enter
Wait+Screen "JK_WORKSPACE_FILES_RETAINED"
Type "exit"
Enter
