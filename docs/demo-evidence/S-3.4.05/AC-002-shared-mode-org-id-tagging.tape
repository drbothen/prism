# AC-002 — Shared-mode X-Prism-Org-Id tagging (BC-3.2.004)
# Traces to: BC-3.2.004 postcondition — OrgId in payload body not routing headers
# Story: S-3.4.05
#
# Runs org_tagging tests across all three crates — shows OrgId UUID is captured
# in the payload body (Slack block kit, PD custom_details, Jira issue field)
# and does NOT appear in HTTP headers or URL path segments.
#
# VHS tape — produces AC-002-shared-mode-org-id-tagging.{gif,webm}

Output docs/demo-evidence/S-3.4.05/AC-002-shared-mode-org-id-tagging.gif
Output docs/demo-evidence/S-3.4.05/AC-002-shared-mode-org-id-tagging.webm

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

# Demo — run org_tagging integration tests for all three crates
Type "cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --test org_tagging --features dtu 2>&1 | grep -E 'org_id|running [0-9]+ test|test result'"
Enter

Wait+Screen /test result/
Sleep 2s
