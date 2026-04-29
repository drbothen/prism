Output docs/demo-evidence/S-3.7.02/BC-3.4.001-004-claroty-generator-24-green.gif
Output docs/demo-evidence/S-3.7.02/BC-3.4.001-004-claroty-generator-24-green.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.02 && clear"
Enter
Sleep 2s
Show

Type "# BC-3.4.001-004 / VP-108,112-114,119-120: Claroty fixture generator — 24/24 GREEN"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty --features fixture-gen --test bc_3_4_claroty_generator 2>&1"
Enter
Sleep 30s
