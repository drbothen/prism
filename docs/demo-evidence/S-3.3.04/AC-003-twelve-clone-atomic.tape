# AC-003: Atomic port pre-allocation — 12-clone startup under 5s (BC-3.5.002 postcondition 5)
#
# Runs test_BC_3_5_002_ac008_twelve_clone_startup_under_5s.
# Demonstrates that 12 clones (3 orgs x 4 sensor types) complete build() within 5s via tokio::join!.

Output AC-003-twelve-clone-atomic.gif
Output AC-003-twelve-clone-atomic.webm

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

Type "# AC-003: 12-clone atomic port pre-allocation completes within 5s budget"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test network_isolation_test test_BC_3_5_002_ac008_twelve_clone_startup_under_5s -- --nocapture 2>&1 | tail -40"
Enter
Sleep 120s
