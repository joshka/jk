Output target/betamax/jk-log.gif
Output target/betamax/jk-log-final-state.json

Require cargo
Require jj

Set Shell "bash"
Set Theme "Aardvark Ink"
Set FontSize 22
Set Width 1200
Set Height 720
Set Margin 12
Set WindowBar Colorful
Set BorderRadius 8
Set CursorBlink false
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"

Hide
Type "repo=${JK_REPO:-$HOME/local/jk}"
Enter
Wait
Type "cd \"$repo\" && test -f Cargo.toml && test -d .jj && pwd"
Enter
Wait+Screen "/local/jk"
Type "cargo build --manifest-path Cargo.toml -p jk >/tmp/jk-betamax-build.log 2>&1 && echo JK_BUILD_READY"
Enter
Wait+Screen "JK_BUILD_READY"
Show

Type "./target/debug/jk -R . -n 3"
Enter
Wait+Screen "jk jj"
Sleep 1s
Screenshot target/betamax/jk-log-initial.png
State target/betamax/jk-log-initial.json

Type "L"
Wait+Screen "jk jj log"
Sleep 300ms
Screenshot target/betamax/jk-log-switched-log.png
State target/betamax/jk-log-switched-log.json

Type "H"
Wait+Screen "/jk jj *\\n/"
Sleep 300ms
Screenshot target/betamax/jk-log-switched-home.png
State target/betamax/jk-log-switched-home.json

Type "j"
Sleep 300ms
Screenshot target/betamax/jk-log-selected-second.png
State target/betamax/jk-log-selected-second.json

Type "k"
Sleep 300ms
Screenshot target/betamax/jk-log-selected-first-again.png
State target/betamax/jk-log-selected-first-again.json

Enter
Sleep 300ms
Screenshot target/betamax/jk-log-expanded.png
State target/betamax/jk-log-expanded.json

Left
Sleep 300ms
Screenshot target/betamax/jk-log-collapsed-left.png
State target/betamax/jk-log-collapsed-left.json

Right
Sleep 300ms
Screenshot target/betamax/jk-log-expanded-right.png
State target/betamax/jk-log-expanded-right.json

Type "r"
Sleep 500ms
Screenshot target/betamax/jk-log-refreshed.png
State target/betamax/jk-log-refreshed.json

Type "q"
Sleep 300ms

Type "./target/debug/jk -R . log -n 3"
Enter
Wait+Screen "jk jj log"
Sleep 1s
Screenshot target/betamax/jk-log-command.png
State target/betamax/jk-log-command.json

Type "q"
Hide
Type "exit"
Enter
