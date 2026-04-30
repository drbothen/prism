# AC-004 — D-048 session-registry per-org isolation (BC-3.2.003)
# Traces to: BC-3.2.003, VP-125
# Story: S-3.4.03
#
# VHS tape — produces AC-004-session-registry-per-org-isolation.{gif,webm}

Output docs/demo-evidence/S-3.4.03/AC-004-session-registry-per-org-isolation.gif
Output docs/demo-evidence/S-3.4.03/AC-004-session-registry-per-org-isolation.webm

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

# Demo — session-registry is per-org; org-A sessions not visible to org-B
Type "cargo test -p prism-dtu-crowdstrike --features dtu --test harness_tests -- test_BC_3_2_003_ac_session_registry_per_org_isolation 2>&1 | tail -6"
Enter
Sleep 15s

Sleep 2s
