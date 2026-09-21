Output target/dogfood-artifacts/betamax/form-style-narrow.gif
Require jj
Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 20
Set Width 820
Set Height 740
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Off
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
Show
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/form-style-narrow-rebase-selector.png
State target/dogfood-artifacts/betamax/form-style-narrow-rebase-selector.json
Enter
Wait+Screen "Source role:"
Type "o"
Wait+Screen "Run options"
Wait+Screen "snapshot/update"
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/form-style-narrow-run-options.png
State target/dogfood-artifacts/betamax/form-style-narrow-run-options.json
Escape
Wait+Screen "Source role:"
Escape
Wait+Screen "1 marked"
Type "a"
Wait+Screen "Actions"
Type "m"
Wait+Screen "Describe revision"
Sleep 500ms
Screenshot target/dogfood-artifacts/betamax/form-style-narrow-describe.png
State target/dogfood-artifacts/betamax/form-style-narrow-describe.json
Escape
Wait+Screen "1 marked"
Hide
Type "q"
Wait
Type@0ms "test \"$(jj -R \"$rebase_repo\" --ignore-working-copy log --no-graph -r @ -T commit_id)\" = \"$initial_commit\" && printf \"JK_%s\\n\" FORMS_NOT_EXECUTED"
Enter
Wait+Screen "JK_FORMS_NOT_EXECUTED"
Type "exit"
Enter
