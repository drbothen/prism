# AC-6: prism-demo.toml loaded and config parsing verified.
#
# Shows config file exists, then drives the AC-6 integration tests which verify
# prism-demo.toml contains all 6 clone port assignments and bare-name credential refs.

Output AC-6-prism-demo-toml.gif
Output AC-6-prism-demo-toml.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.20-dtu-demo-server"
  Enter
  Sleep 500ms
Show

Type "# AC-6: prism-demo.toml config parsing -- all 6 clones, bare-name credential refs"
Enter
Sleep 500ms

Type "ls crates/prism-dtu-demo-server/configs/"
Enter
Sleep 1s

Type "cargo test --test ac_6_prism_demo_toml --features dtu -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
