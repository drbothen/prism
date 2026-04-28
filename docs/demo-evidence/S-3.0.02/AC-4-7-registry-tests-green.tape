Output docs/demo-evidence/S-3.0.02/AC-4-7-registry-tests-green.gif
Output docs/demo-evidence/S-3.0.02/AC-4-7-registry-tests-green.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.0.02 && clear"
Enter
Sleep 2s
Show

Type "# AC-4..7: DTU_DEFAULT_MODE registry -- all 17 tests GREEN"
Enter
Sleep 500ms
Type "cargo test -p prism-core --test bc_3_2_005_dtu_registry 2>&1"
Enter
Sleep 20s
Sleep 2s
