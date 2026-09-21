Output target/dogfood-artifacts/betamax/run-options.gif
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 1400
Set Height 900
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 20ms
Set WaitTimeout 30s

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" workflows"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "initial_commit=$(jj -R \"$rebase_repo\" --ignore-working-copy log --no-graph -r @ -T commit_id) && printf \"JK_%s\\n\" BASELINE_READY"
Enter
Wait+Screen "JK_BASELINE_READY"
Type@0ms "cd \"$rebase_repo\" && clear; \"$bin\" -R \"$rebase_repo\" -n 5"
Enter
Wait+Screen "Rebase source"
Space
Type "j"
Type "R"
Wait+Screen "Choose rebase destination"
Type "/destination"
Enter
Wait+Screen "Source role:"
Show
Caption "Run options apply to this exact command and its fixed repository."
Sleep 650ms
Screenshot target/dogfood-artifacts/betamax/run-options-initial-preview.png
State target/dogfood-artifacts/betamax/run-options-initial-preview.json
Type "o"
Wait+Screen "Run options"
Wait+Screen "snapshot/update"
Sleep 650ms
Screenshot target/dogfood-artifacts/betamax/run-options-drawer.png
State target/dogfood-artifacts/betamax/run-options-drawer.json
Enter
Wait+Screen "ignore"
Caption "Escape discards the draft working-copy policy."
Escape
Wait+Screen "Source role:"
Sleep 650ms
Screenshot target/dogfood-artifacts/betamax/run-options-cancelled-preview.png
State target/dogfood-artifacts/betamax/run-options-cancelled-preview.json
Type "o"
Wait+Screen "snapshot/update"
Enter
Wait+Screen "ignore"
Type "a"
Wait+Screen "Source role:"
Wait+Screen "--ignore-working-copy"
Caption "Apply changes the exact command; Enter is still required to execute it."
Sleep 650ms
Screenshot target/dogfood-artifacts/betamax/run-options-applied-preview.png
State target/dogfood-artifacts/betamax/run-options-applied-preview.json
Escape
Wait+Screen "1 marked"
Wait+Screen "Rebase destination"
Hide
Caption ""
Type "q"
Wait
Type@0ms "test \"$(jj -R \"$rebase_repo\" --ignore-working-copy log --no-graph -r @ -T commit_id)\" = \"$initial_commit\" && printf \"JK_%s\\n\" OPTIONS_NOT_EXECUTED"
Enter
Wait+Screen "JK_OPTIONS_NOT_EXECUTED"
Type "exit"
Enter
