# AC-002 + AC-004 — Red Gate tests PASS (3/3)
# Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
# BC: BC-2.16.013 v1.25 §Postconditions §1
#
# Demonstrates: cargo nextest run for the claroty_spec_prose_fidelity binary
# shows 3 tests PASS: AC-001 no-regression guard, AC-002 red gate, AC-004 functional-fields guard.

Output AC-002-AC-004-red-gate-tests.gif
Output AC-002-AC-004-red-gate-tests.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1100
Set Height 500
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-SPEC-PROSE-FIX-001"
Enter
Sleep 1s
Show

Type "# AC-002 + AC-004: Red Gate tests — 3/3 PASS"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-sensors -E 'binary(claroty_spec_prose_fidelity)'"
Enter
Sleep 30s

Sleep 2s
