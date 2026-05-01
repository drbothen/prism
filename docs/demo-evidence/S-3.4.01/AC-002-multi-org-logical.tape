# AC-002: Multi-org logical isolation — pairwise-disjoint device ID sets
#
# Runs ac_multi_org_logical_isolation: 2-org logical harness returns disjoint
# device ID sets for test-tenant vs other-tenant.
# Traces to: BC-3.5.001 postcondition 2

Output AC-002-multi-org-logical.gif
Output AC-002-multi-org-logical.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 600s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.01"
  Enter
  Sleep 500ms
Show

Type "# AC-002: ac_multi_org_logical_isolation — device IDs pairwise disjoint (S-3.4.01)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-claroty --features dtu --test harness_tests ac_multi_org_logical_isolation -- --nocapture 2>&1"
Enter
Sleep 30s
Sleep 2s
