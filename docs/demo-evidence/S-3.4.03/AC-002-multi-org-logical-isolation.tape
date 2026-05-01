# AC-002 — Multi-org logical isolation (detection store + containment store disjoint per org)
# Traces to: BC-3.5.001 postcondition 2, VP-123
# Story: S-3.4.03
#
# VHS tape — produces AC-002-multi-org-logical-isolation.{gif,webm}

Output docs/demo-evidence/S-3.4.03/AC-002-multi-org-logical-isolation.gif
Output docs/demo-evidence/S-3.4.03/AC-002-multi-org-logical-isolation.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.03"
Enter
Sleep 1s
Show

# Demo — 2-org logical harness: pairwise-disjoint device sets
Type "cargo test -p prism-dtu-crowdstrike --features dtu --test harness_tests -- test_BC_3_5_001_ac_multi_org_logical_isolation 2>&1 | tail -6"
Enter
Sleep 15s

Sleep 2s
