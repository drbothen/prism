# AC-9: Bind security -- loopback binding works; non-loopback rejected without --bind-any.
#
# Drives the AC-9 integration tests: non-loopback config with missing env var returns Err
# citing R-DEMO-001; loopback-only config always allowed.

Output AC-9-bind-security.gif
Output AC-9-bind-security.webm

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

Type "# AC-9: Bind security -- non-loopback rejected; loopback always allowed"
Enter
Sleep 500ms

Type "cargo test --test ac_9_bind_security --features dtu -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
