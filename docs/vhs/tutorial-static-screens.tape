Output target/vhs/tutorial-static-screens.gif

Set Shell "bash"
Set Theme "Aardvark Blue"
Set FontSize 20
Set Width 1280
Set Height 760
Set Padding 24
Set Framerate 24
Set TypingSpeed 70ms

Hide
Type "env -u NO_COLOR CLICOLOR_FORCE=1 COLORTERM=truecolor TERM=xterm-256color ./target/debug/jk"
Enter
Sleep 2.6s
Show
Sleep 1.8s
Screenshot target/vhs/static-log.png

Type "s"
Sleep 1.8s
Screenshot target/vhs/static-status.png

Type "l"
Sleep 1.4s
Type "o"
Sleep 1.8s
Screenshot target/vhs/static-operation-log.png

Type "l"
Sleep 1.4s
Type "L"
Sleep 1.8s
Screenshot target/vhs/static-bookmark-list.png

Type "l"
Sleep 1.4s
Type "v"
Sleep 1.8s
Screenshot target/vhs/static-resolve-list.png

Type "l"
Sleep 1.4s
Type "f"
Sleep 1.8s
Screenshot target/vhs/static-file-list.png

Type "l"
Sleep 1.4s
Type "t"
Sleep 1.8s
Screenshot target/vhs/static-tag-list.png

Type "l"
Sleep 1.4s
Type "w"
Sleep 1.8s
Screenshot target/vhs/static-workspace-root.png

Type "l"
Sleep 1.4s
Type "?"
Sleep 1.8s
Screenshot target/vhs/static-help.png

Type "K"
Sleep 1.8s
Screenshot target/vhs/static-keys.png

Sleep 1.4s
Type "q"
