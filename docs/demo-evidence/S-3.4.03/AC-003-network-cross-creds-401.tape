# AC-003 — Network cross-creds 401 (IsolationMode::Network, cross-org credential mismatch)
# Traces to: BC-3.5.002 postcondition 2, VP-124
# Story: S-3.4.03
#
# VHS tape — produces AC-003-network-cross-creds-401.{gif,webm}

Output docs/demo-evidence/S-3.4.03/AC-003-network-cross-creds-401.gif
Output docs/demo-evidence/S-3.4.03/AC-003-network-cross-creds-401.webm

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

# Demo — cross-org credential mismatch returns HTTP 401 in network isolation mode
Type "cargo test -p prism-dtu-crowdstrike --features dtu --test harness_tests -- test_BC_3_5_002_ac_network_cross_creds_401 2>&1 | tail -6"
Enter
Sleep 15s

Sleep 2s
