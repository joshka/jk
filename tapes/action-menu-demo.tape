Output target/dogfood-artifacts/betamax/action-menu-workflows-concise-20260918.gif

# Treat captions as subtitles: allow reading time plus time to inspect the UI.
# Explain transitions before pressing keys; give dense dialogs longer than simple log states.
# Never show setup output or quit back to a visible shell.
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
Set TypingSpeed 80ms
Set WaitTimeout 60s
Env NO_COLOR "1"

Hide
Type "cd /Users/joshka/local/jk/work/action-menu && cargo build -q -p jk && JK_DEMO_REPO=$(bash scripts/action-menu-demo-fixture.sh) && export JK_DEMO_REPO"
Enter
Wait
Type "clear && exec target/debug/jk -R $JK_DEMO_REPO -n 8"
Enter
Wait+Screen "jk jj"
Wait+Screen "Experiment with forecast caching"
Sleep 2s
Caption "A small forecast service"
Show
Sleep 3s

Type "a"
Wait+Screen "Actions"
Caption "Change actions"
Sleep 6s
Type "m"
Wait+Screen "Describe revision"
Caption "Rename a change"
Sleep 3s
Ctrl+u
Type "Tune forecast cache lifetime"
Sleep 3s
Enter
Wait+Screen "Described revision"
Caption ""
Sleep 3s

Type "a"
Wait+Screen "Actions"
Sleep 1s
Type "m"
Wait+Screen "Describe revision"
Caption "Pick up where you left off"
Sleep 3s
Screenshot target/dogfood-artifacts/betamax/action-menu-concise-describe-20260918.png
Type " safely"
Sleep 2s
Enter
Wait+Screen "Described revision"
Caption ""
Sleep 3s

Caption "Create a child change"
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "n"
Wait+Screen "Created new change"
Sleep 4s
Screenshot target/dogfood-artifacts/betamax/action-menu-concise-new-20260918.png

Caption "Abandon an empty change"
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "a"
Wait+Screen "Abandoned revision"
Sleep 3s
Caption "jj leaves a fresh empty working copy"
Sleep 4s
Screenshot target/dogfood-artifacts/betamax/action-menu-concise-empty-20260918.png

Caption "Abandon the cache experiment"
Type "j"
Sleep 3s
Type "a"
Wait+Screen "Actions"
Sleep 2s
Type "a"
Wait+Screen "Confirm command"
Caption "This change contains work. Confirm before removing it."
Sleep 8s
Screenshot target/dogfood-artifacts/betamax/action-menu-concise-confirm-20260918.png
Enter
Wait+Screen "Abandoned revision"
Caption "Experiment removed. Earlier work intact."
Sleep 4s
Screenshot target/dogfood-artifacts/betamax/action-menu-concise-finished-20260918.png
Hide
Caption ""
Type "q"
