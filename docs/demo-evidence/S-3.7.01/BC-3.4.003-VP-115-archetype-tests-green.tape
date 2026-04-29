Output docs/demo-evidence/S-3.7.01/BC-3.4.003-VP-115-archetype-tests-green.gif
Output docs/demo-evidence/S-3.7.01/BC-3.4.003-VP-115-archetype-tests-green.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.01 && clear"
Enter
Sleep 2s
Show

Type "# BC-3.4.003 / VP-115: Archetype catalog + GenOpts — 39 tests GREEN"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-common --features fixture-gen --test bc_3_4_001_003_archetype_genopts 2>&1"
Enter
Sleep 15s
