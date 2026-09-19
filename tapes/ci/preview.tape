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
Set KeyboardOverlay Off
Set TypingSpeed 20ms
Set WaitTimeout 30s
Env NO_COLOR "1"
Env JJ_CONFIG "/dev/null"

Hide
Type@0ms "cd \"$BETAMAX_WORKING_DIRECTORY\" && binary=\"$PWD/target/debug/jk\" && cd \"$(cat target/betamax-preview/repository)\" && \"$binary\" log -n 8"
Enter
Wait+Screen "Improve greeting"
Sleep 800ms
Show
Screenshot target/betamax-preview/log.png
Sleep 1s

Type "?"
Wait+Screen "Open and inspect:"
Sleep 1200ms
Screenshot target/betamax-preview/log-help.png
Escape
Sleep 300ms

Type "d"
Wait+Screen "jk jj diff"
Wait+Screen "Review changes from your terminal."
Sleep 1200ms
Screenshot target/betamax-preview/diff.png

Type "?"
Wait+Screen "Move and find:"
Sleep 1200ms
Screenshot target/betamax-preview/diff-help.png
Escape
Sleep 300ms
Type "L"
Wait+Screen "Improve greeting"
Sleep 800ms
Hide
Type "q"
Sleep 300ms
Type "exit"
Enter
