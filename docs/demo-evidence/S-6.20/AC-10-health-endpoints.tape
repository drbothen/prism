# AC-10: /dtu/health responds 200 on all 6 clone ports.
#
# Drives the AC-10 integration tests which start all 6 clones and GET /dtu/health
# on each port, asserting HTTP 200 from every clone.

Output AC-10-health-endpoints.gif
Output AC-10-health-endpoints.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.20-dtu-demo-server"
  Enter
  Sleep 500ms
Show

Type "# AC-10: /dtu/health returns 200 on all 6 clone ports"
Enter
Sleep 500ms

Type "cargo test --test ac_10_health_endpoints --features dtu -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
