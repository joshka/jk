Output target/dogfood-artifacts/betamax/workspace-lifecycle-20260919.gif

Require cargo
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1200
Set Height 720
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set KeyboardOverlay Input
Set TypingSpeed 55ms
Set WaitTimeout 60s
Env NO_COLOR "1"

Hide
Type "cd \"${JK_SOURCE_REPO:?Run just betamax-workspace-lifecycle}\" && fixture_root=$(bash scripts/workflows-demo-fixture.sh target/dogfood-artifacts) && JK_DEMO_REPO=$fixture_root/workspaces/repo && printf 'JK_%s\\n' WORKSPACES_READY"
Enter
Wait+Screen "JK_WORKSPACES_READY"
Type "clear && exec target/debug/jk -R \"$JK_DEMO_REPO\" workspaces"
Enter
Wait+Screen "jj workspace list"
Wait+Screen "scratch"
Show
Caption "Workspace lifecycle actions live together"
Sleep 3s
Type "j"
Type "a"
Wait+Screen "Workspace lifecycle"
Sleep 5s
Screenshot target/dogfood-artifacts/betamax/workspace-lifecycle-menu-20260919.png

Caption "Forget says exactly what changes—and what does not"
Type "f"
Wait+Screen "Files are not deleted"
Sleep 7s
Screenshot target/dogfood-artifacts/betamax/workspace-lifecycle-forget-20260919.png
Type "n"
Wait+Screen "scratch"
Caption "Cancel returns to the same usable list"
Sleep 3s

Caption "Add collects a destination, then reviews exact argv"
Type "a"
Type "a"
Wait+Screen "Destination path"
Type "../demo-added"
Enter
Wait+Screen "Exact command"
Sleep 6s
Screenshot target/dogfood-artifacts/betamax/workspace-lifecycle-add-20260919.png
Enter
Wait+Screen "demo-added"
Sleep 4s

Caption "Rename is scoped to the selected workspace"
Type "a"
Type "r"
Wait+Screen "New workspace name"
Type "renamed-demo"
Enter
Wait+Screen "workspace rename"
Sleep 5s
Enter
Wait+Screen "renamed-demo"
Sleep 4s

Caption "Forget removes metadata; the directory and files stay on disk"
Type "a"
Type "f"
Wait+Screen "Files are not deleted"
Sleep 6s
Type "y"
Wait+Screen "default"
Sleep 5s
Screenshot target/dogfood-artifacts/betamax/workspace-lifecycle-finished-20260919.png
Hide
Caption ""
Type "q"
