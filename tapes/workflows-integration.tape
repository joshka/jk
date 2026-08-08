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
Set KeyboardOverlay Input
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type "cd \"${JK_SOURCE_REPO:?Run just betamax-workflows}\" && source_repo=$PWD && bin=$PWD/target/debug/jk && test -x \"$bin\" && fixture_root=$(bash scripts/workflows-demo-fixture.sh target/dogfood-artifacts) && printf 'JK_%s\\n' FIXTURES_READY"
Enter
Wait+Screen "JK_FIXTURES_READY"
Type "rebase_repo=$fixture_root/rebase && squash_repo=$fixture_root/squash && restore_repo=$fixture_root/restore && workspaces_repo=$fixture_root/workspaces/repo"
Enter
Wait
Type "clear"
Enter
Show

Caption "Rebase previews an exact destination, and Escape leaves the graph unchanged."
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
Type "/destination"
Enter
Wait+Screen "Confirm command"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-preview.png
Escape
Wait+Screen "Rebase destination"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-cancelled.png

Hide
Type "q"
Wait
Type "test \"$(jj -R \"$rebase_repo\" log --no-graph -r @ -T 'parents.map(|x| x.description().first_line()).join(\",\")')\" = \"Rebase base\" && printf 'JK_%s\\n' REBASE_CANCELLED"
Enter
Wait+Screen "JK_REBASE_CANCELLED"

Caption "Confirming the same rebase refreshes the log and records its resulting operation."
Show
Type "\"$bin\" -R \"$rebase_repo\" -n 5"
Enter
Wait+Screen "Rebase source"
Type " "
Type "j"
Type "R"
Wait+Screen "Choose rebase destination"
Enter
Wait+Screen "Confirm command"
Enter
Wait+Screen "u undo"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-success.png
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-history.png
Type "o"
Wait+Screen "Changed commits"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-rebase-operation.png
Hide
Type "q"
Wait
Type "test \"$(jj -R \"$rebase_repo\" log --no-graph -r @ -T 'parents.map(|x| x.description().first_line()).join(\",\")')\" = \"Rebase destination\" && printf 'JK_%s\\n' REBASE_SUCCESS"
Enter
Wait+Screen "JK_REBASE_SUCCESS"

Caption "Squash preserves marked sources and the destination cursor when its preview is cancelled."
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
Wait+Screen "Details"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-preview.png
Escape
Wait+Screen "Squash destination"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-cancelled.png

Caption "The confirmed squash has an auditable Command History entry and operation link."
Type "a"
Type "s"
Wait+Screen "Details"
Enter
Wait+Screen "Squashed source changes"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-success.png
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-history.png
Type "o"
Wait+Screen "Changed commits"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-squash-operation.png
Hide
Type "q"
Wait

Caption "Restore names the source, working-copy destination, and destructive all-path scope."
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
Escape
Wait+Screen "Restore base"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-cancelled.png

Caption "Confirming restore refreshes the working copy and captures its recovery operation."
Type "a"
Type "r"
Wait+Screen "Affected content: all paths"
Enter
Wait+Screen "Restored all paths into working copy"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-success.png
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-history.png
Type "o"
Wait+Screen "Changed commits"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-restore-operation.png
Hide
Type "q"
Wait

Caption "Workspace lifecycle keeps add and forget together, with a cancel-safe confirmation."
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
Type "n"
Wait+Screen "scratch"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-cancelled.png

Caption "Adding a workspace confirms exact argv, refreshes the list, and records an operation."
Type "a"
Type "a"
Wait+Screen "Destination path"
Type "../one/demo-added"
Enter
Wait+Screen "Exact command"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-add-preview.png
Enter
Wait+Screen "demo-added"
Sleep 900ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-success.png

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

Caption "Confirming forget removes metadata but retains the workspace directory and files."
Type "a"
Type "f"
Wait+Screen "Files are not deleted"
Type "y"
Wait+Screen "default"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-forgotten.png
Type "C"
Wait+Screen "Command History"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/workflows-workspace-history.png
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
Caption ""
Hide
Type "q"
Wait
Type "test -d \"$fixture_root/workspaces/one/demo-added\" && printf 'JK_%s\\n' WORKSPACE_FILES_RETAINED"
Enter
Wait+Screen "JK_WORKSPACE_FILES_RETAINED"
Type "exit"
Enter
