# AC-004: Drop releases ports — ConnectionRefused within 1s (BC-3.5.002 postcondition 6, TV-6)
#
# Runs test_BC_3_5_002_ac005_drop_releases_ports.
# Demonstrates that after drop(harness) all TCP listeners are released — TcpStream::connect returns
# ConnectionRefused confirming no dangling listeners.

Output AC-004-drop-releases-ports.gif
Output AC-004-drop-releases-ports.webm

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

Type "# AC-004: Drop releases all TCP listeners — ConnectionRefused within 1s of drop"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test network_isolation_test test_BC_3_5_002_ac005_drop_releases_ports -- --nocapture 2>&1 | tail -40"
Enter
Sleep 120s
