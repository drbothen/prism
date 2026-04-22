Output docs/demo-evidence/S-1.02/AC-4-credential-path-traversal.gif
Output docs/demo-evidence/S-1.02/AC-4-credential-path-traversal.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 16
Set Padding 24

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.02-entity-types"
Enter
Sleep 1s
Show

Type "cargo test -p prism-core test_BC_S_02_003_ac4_rejects_path_traversal_double_dot"
Enter
Sleep 8s

Sleep 2s
