# AC-002 — Gap-CL-006 CLOSED closure comment present in audit_logs block
# Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
# BC: BC-2.16.013 v1.25 §Postconditions §1
#
# Demonstrates: the audit_logs table block in claroty.sensor.toml contains
# the required "Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001." comment line.

Output AC-002-gap-cl-006-closed-comment.gif
Output AC-002-gap-cl-006-closed-comment.webm

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

Type "# AC-002: Gap-CL-006 CLOSED comment present in audit_logs block"
Enter
Sleep 500ms

Type "rg -n -A2 'table_name = .audit_logs.' crates/prism-sensors/specs/claroty.sensor.toml"
Enter
Sleep 1500ms

Sleep 2s
