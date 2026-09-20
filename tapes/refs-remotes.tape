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
Set Margin 20
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set KeyboardOverlay Input
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type 'repo=$(mktemp -d /tmp/jk-refs-demo.XXXXXX) && jj git init --no-colocate "$repo/client" && jj git init --no-colocate "$repo/remote" && jj -R "$repo/client" bookmark create demo && jj -R "$repo/client" git remote add fixture "$repo/remote/.jj/repo/store/git" && echo JK_REFS_REMOTES_READY'
Enter
Wait+Screen "JK_REFS_REMOTES_READY"
Type 'cargo build --manifest-path /Users/joshka/local/jk/work/refs-remotes/Cargo.toml -p jk >/tmp/jk-refs-remotes-borderless-build.log 2>&1 && echo JK_BUILD_READY'
Enter
Wait+Screen "JK_BUILD_READY"
Type 'clear; /Users/joshka/local/jk/work/refs-remotes/target/debug/jk -R "$repo/client"'
Enter
Wait+Screen "jk jj"
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
Type "x"
Wait+Screen "jj bookmark delete"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-delete.png
Escape
Type "F"
Wait+Screen "Fetch from remote"
Sleep 1400ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-chooser.png
Enter
Wait+Screen "jj git fetch"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-fetch-preview.png
Enter
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-fetch-result.png
Backspace
Type "P"
Wait+Screen "Push dry-run"
Sleep 1200ms
Enter
Wait+Screen "dry-run"
Sleep 2000ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-borderless-push-preview.png
Escape
Escape
Type "q"
Caption ""
Hide
Type "exit"
Enter
