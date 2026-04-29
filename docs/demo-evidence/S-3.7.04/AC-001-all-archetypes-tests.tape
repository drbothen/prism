Output docs/demo-evidence/S-3.7.04/AC-001-all-archetypes-tests.gif
Output docs/demo-evidence/S-3.7.04/AC-001-all-archetypes-tests.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.04 && clear"
Enter
Sleep 2s
Show

Type "# S-3.7.04: BC-3.4.001-004 — Armis generator: all archetypes + determinism + schema"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-armis --features fixture-gen --test bc_3_4_armis_generator 2>&1"
Enter
Sleep 30s
