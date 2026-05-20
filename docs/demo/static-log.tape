Output target/vhs/static-log.gif

Require cargo
Require jj

Set Shell "bash"
Set Width 1200
Set Height 720
Set Theme "Aardvark Blue"

Hide
Type "cd target/demo-repos/static-log"
Enter
Type "cargo run --quiet --manifest-path ../../../Cargo.toml --"
Enter
Sleep 3s
Show

Sleep 1s
Type "j"
Sleep 1s
Type "j"
Sleep 1s
Type "s"
Sleep 2s
Type "h"
Sleep 1s
Type "q"
