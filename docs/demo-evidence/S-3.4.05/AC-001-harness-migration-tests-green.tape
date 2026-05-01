# AC-001 — Slack/PagerDuty/Jira harness migration tests green (all 3 crates)
# Traces to: BC-3.5.001 postcondition 1 (VP-087, VP-088, VP-089)
# Story: S-3.4.05
#
# Shows all three crates' harness_tests.rs passing under IsolationMode::Logical
# with mode = "shared": 24 Slack + 28 PagerDuty + 39 Jira = 91 tests.
#
# VHS tape — produces AC-001-harness-migration-tests-green.{gif,webm}

Output docs/demo-evidence/S-3.4.05/AC-001-harness-migration-tests-green.gif
Output docs/demo-evidence/S-3.4.05/AC-001-harness-migration-tests-green.webm

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
Set WaitTimeout 180s

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.05"
Enter
Sleep 1s
Show

# Demo — run harness_tests for all three crates; filter to key result lines
Type "cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --test harness_tests --features dtu 2>&1 | grep -E 'running [0-9]+ test|test result'"
Enter

Wait+Screen /test result/
Sleep 2s
