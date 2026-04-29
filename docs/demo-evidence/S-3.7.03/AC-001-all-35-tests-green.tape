Output docs/demo-evidence/S-3.7.03/AC-001-all-35-tests-green.gif
Output docs/demo-evidence/S-3.7.03/AC-001-all-35-tests-green.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1400
Set Height 800
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.03 && clear"
Enter
Sleep 2s
Show

Type "# BC-3.4.001/002/004 — 8 archetypes x 4 endpoints — 35 tests GREEN"
Enter
Sleep 300ms
Type "cargo test -p prism-dtu-cyberint --features fixture-gen --test bc_3_4_cyberint_generator 2>&1 | tail -42"
Enter
Sleep 8s
