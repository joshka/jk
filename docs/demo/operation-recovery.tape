Output target/vhs/operation-recovery.gif

Require cargo
Require jj

Set Shell "bash"
Set Width 1200
Set Height 720
Set Theme "Aardvark Blue"

Hide
Type "cd target/demo-repos/operation-recovery"
Enter
Type "cargo run --quiet --manifest-path ../../../Cargo.toml -- operation-log"
Enter
Sleep 3s
Show

Sleep 1s
Type "u"
Sleep 2s
Type "q"
