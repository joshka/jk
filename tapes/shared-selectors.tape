Output target/dogfood-artifacts/betamax/shared-selectors-regions-20260919.gif

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
Set TypingSpeed 60ms
Set WaitTimeout 60s
Env NO_COLOR "1"

Hide
Type "cd /Users/joshka/local/jk/work/shared-selectors && cargo build -q -p jk && SELECTOR_DEMO_REPO=$(bash scripts/shared-selector-demo-fixture.sh) && export SELECTOR_DEMO_REPO"
Enter
Wait
Type "clear && exec target/debug/jk -R $SELECTOR_DEMO_REPO -n 8"
Enter
Wait+Screen "jk jj"
Wait+Screen "Configure request timeouts"
Show

Caption "Filesets resolve one visible file target without changing diff navigation state."
Type "d"
Wait+Screen "jk jj diff"
Sleep 800ms
Type "f"
Wait+Screen "src/timeout.js"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/shared-selectors-fileset-regions-20260919.png
Type "j"
Sleep 500ms
Enter
Sleep 1200ms

Caption "Revision marks preserve the order used for parent roles."
Type "L"
Wait+Screen "jk jj"
Sleep 600ms
Type " "
Sleep 500ms
Type "j"
Sleep 500ms
Type " "
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/shared-selectors-revisions-regions-20260919.png
Type "a"
Wait+Screen "Actions"
Type "n"
Wait+Screen "Created new change"
Sleep 1800ms

Caption "Operation actions resolve the selected full operation ID before opening details."
Type "o"
Wait+Screen "jj op log"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/shared-selectors-operations-regions-20260919.png
Enter
Wait+Screen "jj op show"
Sleep 2200ms
Screenshot target/dogfood-artifacts/betamax/shared-selectors-operation-show-regions-20260919.png

Hide
Caption ""
Type "q"
