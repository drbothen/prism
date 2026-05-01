# AC-004 — Client-mode override does not produce startup error (BC-3.3.001)
# Traces to: BC-3.5.001 precondition 2; BC-3.3.001-startup EC-003
# Story: S-3.4.05
#
# Runs ac_client_mode_override_does_not_produce_startup_error test across all
# three harness_tests suites — MSSP Coordination types permit mode="client"
# override without a startup error.
#
# VHS tape — produces AC-004-client-mode-override-no-startup-error.{gif,webm}

Output docs/demo-evidence/S-3.4.05/AC-004-client-mode-override-no-startup-error.gif
Output docs/demo-evidence/S-3.4.05/AC-004-client-mode-override-no-startup-error.webm

Require bash
Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms
Set WaitTimeout 120s

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.05"
Enter
Sleep 1s
Show

# Demo — run client-mode-override test across all three crates
Type "cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --features dtu ac_client_mode_override_does_not_produce_startup_error 2>&1 | grep -E 'client_mode|test result'"
Enter

Wait+Screen /test result/
Sleep 2s
