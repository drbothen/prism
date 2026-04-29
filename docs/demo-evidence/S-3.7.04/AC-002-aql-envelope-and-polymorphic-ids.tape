Output docs/demo-evidence/S-3.7.04/AC-002-aql-envelope-and-polymorphic-ids.gif
Output docs/demo-evidence/S-3.7.04/AC-002-aql-envelope-and-polymorphic-ids.webm

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

Type "# S-3.7.04: AC-003 AQL envelope + AC-004 EC-001 polymorphic integer asset IDs"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-armis --features fixture-gen --test bc_3_4_armis_generator aql_envelope every_fifth 2>&1 -- --nocapture"
Enter
Sleep 20s
