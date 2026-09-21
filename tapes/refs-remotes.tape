Output target/dogfood-artifacts/betamax/refs-remotes-borderless.gif
Output target/dogfood-artifacts/betamax/refs-remotes-borderless-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1200
Set Height 720
Set Padding 0
Set Margin 0
Set WindowBar ""
Set BorderRadius 0
Set CursorBlink false
Set Framerate 15
Set KeyboardOverlay Input
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" refs"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear; \"$bin\" -R \"$fixture\" -n 8"
Enter
Wait+Screen "Prepare remote workflow"
Show
Caption "Inspect bookmarks, then choose a named remote and confirm fetch."
Type "B"
Wait+Screen "jj bookmark list"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-bookmarks.png
State target/dogfood-artifacts/betamax/refs-remotes-borderless-bookmarks.json
Type "c"
Wait+Screen "Bookmark mutation"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-create.png
State target/dogfood-artifacts/betamax/refs-remotes-borderless-create.json
Type "review-topic"
Sleep 800ms
Enter
Wait+Screen "jj bookmark create"
Sleep 1800ms
Escape
Wait+Screen "Bookmark mutation"
Escape
Type "x"
Wait+Screen "jj bookmark delete"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-delete.png
State target/dogfood-artifacts/betamax/refs-remotes-borderless-delete.json
Escape
Type "F"
Wait+Screen "Fetch from remote"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-chooser.png
State target/dogfood-artifacts/betamax/refs-remotes-borderless-chooser.json
Enter
Wait+Screen "jj git fetch"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-fetch-preview.png
State target/dogfood-artifacts/betamax/refs-remotes-borderless-fetch-preview.json
Enter
Wait+Screen "Status: success"
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-fetch-result.png
State target/dogfood-artifacts/betamax/refs-remotes-borderless-fetch-result.json
Backspace
Type "P"
Wait+Screen "Push dry-run"
Sleep 1200ms
Enter
Wait+Screen "dry-run"
Sleep 2000ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-push-preview.png
State target/dogfood-artifacts/betamax/refs-remotes-borderless-push-preview.json
Escape
Escape
Type "q"
Caption ""
Hide
Type "exit"
Enter
