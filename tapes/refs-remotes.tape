Output target/dogfood-artifacts/betamax/refs-remotes.gif
Output target/dogfood-artifacts/betamax/refs-remotes-final-state.json

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
Type 'repo=$(mktemp -d) && jj git init --no-colocate "$repo/client" && jj git init --no-colocate "$repo/remote" && jj -R "$repo/client" bookmark create demo && jj -R "$repo/client" git remote add fixture "$repo/remote/.jj/repo/store/git" && echo JK_REFS_REMOTES_READY'
Enter
Wait+Screen "JK_REFS_REMOTES_READY"
Type 'cargo build --manifest-path /Users/joshka/local/jk/work/refs-remotes/Cargo.toml -p jk >/tmp/jk-refs-remotes-build.log 2>&1 && echo JK_BUILD_READY'
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
Screenshot target/dogfood-artifacts/betamax/refs-remotes-bookmarks.png
State target/dogfood-artifacts/betamax/refs-remotes-bookmarks.json
Type "c"
Wait+Screen "Bookmark mutation"
Sleep 600ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-create.png
State target/dogfood-artifacts/betamax/refs-remotes-create.json
Escape
Type "F"
Wait+Screen "Fetch from remote"
Screenshot target/dogfood-artifacts/betamax/refs-remotes-chooser.png
Enter
Wait+Screen "jj git fetch"
Screenshot target/dogfood-artifacts/betamax/refs-remotes-fetch-preview.png
Enter
Sleep 800ms
Screenshot target/dogfood-artifacts/betamax/refs-remotes-fetch-result.png
Backspace
Type "P"
Wait+Screen "Push dry-run"
Enter
Wait+Screen "dry-run"
Screenshot target/dogfood-artifacts/betamax/refs-remotes-push-preview.png
Escape
Escape
Type "q"
Caption ""
Hide
Type "exit"
Enter
