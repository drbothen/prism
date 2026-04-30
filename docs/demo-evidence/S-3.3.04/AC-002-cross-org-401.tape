# AC-002: Cross-org HTTP 401 routing (BC-3.5.002 postcondition 2, TV-3, EC-001)
#
# Runs test_BC_3_5_002_ac004_cross_org_credential_mismatch_returns_401.
# Demonstrates that OrgA credentials routed to OrgB endpoint return HTTP 401.

Output AC-002-cross-org-401.gif
Output AC-002-cross-org-401.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 120s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.04"
  Enter
  Sleep 500ms
Show

Type "# AC-002: Cross-org credential mismatch returns HTTP 401 (not 200, not silent)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test network_isolation_test test_BC_3_5_002_ac004_cross_org_credential_mismatch_returns_401 -- --nocapture 2>&1 | tail -40"
Enter
Sleep 120s
