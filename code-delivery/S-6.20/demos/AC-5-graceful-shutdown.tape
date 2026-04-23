# AC-5: Graceful shutdown — SIGINT within 5s stops all clones; ports are released.
#
# Drives the AC-5 integration tests which start all 6 clones, call stop_all(),
# and assert all ports are released within 500ms (no listener leak).

Output AC-5-graceful-shutdown.gif
Output AC-5-graceful-shutdown.webm

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

Type "# AC-5: Graceful shutdown -- all 6 clones stop within 5s, ports released"
Enter
Sleep 500ms

Type "cargo test --test ac_5_graceful_shutdown --features dtu -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
