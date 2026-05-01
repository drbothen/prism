# AC-003: Network isolation cross-creds 401
#
# Runs ac_network_cross_creds_401: 2-org network harness returns HTTP 401
# when test-tenant's token is sent to other-tenant's endpoint.
# Traces to: BC-3.5.002 postcondition 2

Output AC-003-network-cross-creds-401.gif
Output AC-003-network-cross-creds-401.webm

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

Type "# AC-003: ac_network_cross_creds_401 — cross-org token rejected 401 (S-3.4.01)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-claroty --features dtu --test harness_tests ac_network_cross_creds_401 -- --nocapture 2>&1"
Enter
Sleep 30s
Sleep 2s
