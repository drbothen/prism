# AC-001 — No stale "DTU gap" comments in claroty.sensor.toml audit_logs block
# Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
# BC: BC-2.16.013 v1.25 §Postconditions §1

Output AC-001-no-stale-dtu-gap-comments.gif
Output AC-001-no-stale-dtu-gap-comments.webm

Require rg

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 360
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-SPEC-PROSE-FIX-001"
Enter
Sleep 1s
Show

Type "# AC-001: no stale DTU gap comments in audit_logs (exit 1 = no matches = PASS)"
Enter
Sleep 500ms

Type "rg -c 'DTU gap' crates/prism-sensors/specs/claroty.sensor.toml || echo PASS-no-matches"
Enter
Sleep 1500ms

Type "rg -c '404 until DTU route lands' crates/prism-sensors/specs/claroty.sensor.toml || echo PASS-no-matches"
Enter
Sleep 1500ms

Sleep 2s
