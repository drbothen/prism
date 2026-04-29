Output docs/demo-evidence/S-3.2.02/AC-002-proptest-invariants.gif
Output docs/demo-evidence/S-3.2.02/AC-002-proptest-invariants.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.02 && clear"
Enter
Sleep 2s
Show

Type "# S-3.2.02 BC-3.2.001: proptest invariants — 3 proptests x 1000 cases each"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-armis --features dtu --test multi_tenant prop_ -- --nocapture 2>&1"
Enter
Sleep 30s
