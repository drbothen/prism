# AC-003 — Multi-org logical isolation in shared mode
# Traces to: BC-3.5.001 (VP-090, VP-122) — logical isolation invariants
# Story: S-3.4.05
#
# Shows ac_multi_org_logical_isolation_shared_mode test passing in all three
# harness_tests suites — verifies payloads from distinct OrgIds are not mixed.
#
# VHS tape — produces AC-003-multi-org-logical-isolation.{gif,webm}

Output docs/demo-evidence/S-3.4.05/AC-003-multi-org-logical-isolation.gif
Output docs/demo-evidence/S-3.4.05/AC-003-multi-org-logical-isolation.webm

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

# Demo — filter to the isolation-specific test across all three crates
Type "cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --features dtu ac_multi_org_logical_isolation_shared_mode 2>&1 | grep -E 'ac_multi_org|test result'"
Enter

Wait+Screen /test result/
Sleep 2s
