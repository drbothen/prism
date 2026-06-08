# AC-003 — BC-2.16.013 §Postconditions §1 audit_logs prose verified
# Story: S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
# BC: BC-2.16.013 v1.25 §Postconditions §1
#
# Demonstrates: the BC-2.16.013 file contains the corrected audit_logs postcondition
# prose: "POST /api/v1/audit_log/get; DTU route registered by
# S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED)".

Output AC-003-bc-postcondition-prose.gif
Output AC-003-bc-postcondition-prose.webm

Require rg

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 400
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/jmagady/Dev/prism"
Enter
Sleep 1s
Show

Type "# AC-003: BC-2.16.013 v1.25 audit_logs postcondition prose (Gap-CL-006 CLOSED)"
Enter
Sleep 500ms

Type "rg -n 'Gap-CL-006 CLOSED' .factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
Enter
Sleep 1500ms

Sleep 2s
