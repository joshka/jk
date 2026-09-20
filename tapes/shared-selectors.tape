Output target/dogfood-artifacts/betamax/shared-selectors-review-20260920.gif
Output target/dogfood-artifacts/betamax/shared-selectors-review-20260920-final.json

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
Type "cd \"${JK_REPO:?Set JK_REPO to the jk checkout}\" && test -f crates/jk/Cargo.toml && CARGO_BUILD_JOBS=3 cargo build -q -p jk && SELECTOR_DEMO_REPO=$(bash scripts/shared-selector-demo-fixture.sh) && export SELECTOR_DEMO_REPO && echo JK_FIXTURE_READY"
Enter
Wait+Screen /(?m)^JK_FIXTURE_READY$/
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
Screenshot target/dogfood-artifacts/betamax/shared-selectors-fileset-review-20260920.png
State target/dogfood-artifacts/betamax/shared-selectors-fileset-review-20260920.json
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
Screenshot target/dogfood-artifacts/betamax/shared-selectors-revisions-review-20260920.png
State target/dogfood-artifacts/betamax/shared-selectors-revisions-review-20260920.json
Type "a"
Wait+Screen "Actions"
Type "n"
Wait+Screen "Created new change"
Sleep 1800ms

Caption "Operation actions resolve the selected full operation ID before opening details."
Type "o"
Wait+Screen "jj op log"
Sleep 1800ms
Screenshot target/dogfood-artifacts/betamax/shared-selectors-operations-review-20260920.png
State target/dogfood-artifacts/betamax/shared-selectors-operations-review-20260920.json
Enter
Wait+Screen "jj op show"
Sleep 2200ms
Screenshot target/dogfood-artifacts/betamax/shared-selectors-operation-show-review-20260920.png
State target/dogfood-artifacts/betamax/shared-selectors-operation-show-review-20260920.json

Hide
Caption ""
Type "q"
