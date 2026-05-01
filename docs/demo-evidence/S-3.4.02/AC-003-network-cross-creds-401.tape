# AC-003 — Network cross-credentials → HTTP 401
# Traces to: S-3.4.02 AC-003, BC-3.5.002
# Story: S-3.4.02
#
# VHS tape — produces AC-003-network-cross-creds-401.{gif,webm}

Output docs/demo-evidence/S-3.4.02/AC-003-network-cross-creds-401.gif
Output docs/demo-evidence/S-3.4.02/AC-003-network-cross-creds-401.webm

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

# Demo: run only the network cross-credentials 401 test (test_BC_3_5_002_ac_network_cross_creds_401)
Type "bash docs/demo-evidence/S-3.4.02/ac-003-run.sh"
Enter
Sleep 15s

Sleep 2s
