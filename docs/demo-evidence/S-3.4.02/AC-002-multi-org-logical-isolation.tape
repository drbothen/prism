# AC-002 — Multi-org logical isolation (2-org disjoint device sets)
# Traces to: S-3.4.02 AC-002, BC-3.5.001 postcondition (multi-org)
# Story: S-3.4.02
#
# VHS tape — produces AC-002-multi-org-logical-isolation.{gif,webm}

Output docs/demo-evidence/S-3.4.02/AC-002-multi-org-logical-isolation.gif
Output docs/demo-evidence/S-3.4.02/AC-002-multi-org-logical-isolation.webm

Require bash

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.02"
Enter
Sleep 1s
Show

# Demo: run only the multi-org isolation test (test_BC_3_5_001_ac_multi_org_logical_isolation)
Type "bash docs/demo-evidence/S-3.4.02/ac-002-run.sh"
Enter
Sleep 15s

Sleep 2s
