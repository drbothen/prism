# AC-3: POST /dtu/configure accepted on each clone.
#
# Drives the AC-3 integration tests which start clones and POST to /dtu/configure,
# verifying the endpoint accepts config changes for CrowdStrike and a L2 clone.

Output AC-3-configure-endpoint.gif
Output AC-3-configure-endpoint.webm

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

Type "# AC-3: POST /dtu/configure accepted on crowdstrike and cyberint (L2) clones"
Enter
Sleep 500ms

Type "cargo test --test ac_3_configure_endpoint --features dtu -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
