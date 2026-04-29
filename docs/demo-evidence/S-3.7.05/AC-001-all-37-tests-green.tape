Output docs/demo-evidence/S-3.7.05/AC-001-all-37-tests-green.gif
Output docs/demo-evidence/S-3.7.05/AC-001-all-37-tests-green.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.05 && clear"
Enter
Sleep 2s
Show

Type "# BC-3.4.001-004 + 8 archetypes + 2-step pagination + OAuth2 + VP-108/112-114/119-121: 37 GREEN"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-crowdstrike --features fixture-gen --test bc_3_4_crowdstrike_generator 2>&1"
Enter
Sleep 25s
