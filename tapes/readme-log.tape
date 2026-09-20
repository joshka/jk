Output target/dogfood-artifacts/readme-media/jk-log-v3.gif
Output target/dogfood-artifacts/betamax/readme-log-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set Width 1200
Set Height 720
Set Padding 0
Set Margin 20
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set KeyboardOverlay Off
Set TypingSpeed 24ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type@0ms "source \"${JK_SOURCE_REPO:-$PWD}/scripts/betamax-setup.sh\" inspection"
Enter
Wait+Screen "JK_FIXTURE_READY"
Type@0ms "clear"
Enter
Wait+Line
Ctrl+L
Sleep 200ms

Type "\"$bin\" -R \"$fixture\" -n 8"
Enter
Wait+Screen "jk jj"
Sleep 1s
Show
Screenshot target/dogfood-artifacts/readme-media/jk-log-v3.png
State target/dogfood-artifacts/readme-media/jk-log-v3.json

Hide
Type "?"
Sleep 500ms
Screenshot target/dogfood-artifacts/readme-media/jk-log-help-v3.png
State target/dogfood-artifacts/readme-media/jk-log-help-v3.json

Type "?"
Sleep 300ms
Show
Type "j"
Sleep 450ms
Type "j"
Sleep 700ms
Screenshot target/dogfood-artifacts/readme-media/jk-log-selection-v3.png
State target/dogfood-artifacts/readme-media/jk-log-selection-v3.json

Enter
Sleep 700ms
Screenshot target/dogfood-artifacts/readme-media/jk-log-expanded-v3.png
State target/dogfood-artifacts/readme-media/jk-log-expanded-v3.json

Type "d"
Wait+Screen "jk jj diff"
Sleep 1200ms
Type "}"
Sleep 700ms
Type "-"
Sleep 1s
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 180ms
Type "j"
Sleep 900ms
Type "+"
Sleep 800ms
Type "H"
Wait+Screen "jk jj"
Sleep 900ms
Hide
Type "q"
Sleep 400ms
Type "exit"
Enter
