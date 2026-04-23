# AC-7: Determinism -- same seed=42, same request sequence yields identical fixture bytes.
#
# Drives the AC-7 integration tests which start two harness instances with seed=42,
# send identical requests, and assert byte-identical response bodies.

Output AC-7-determinism.gif
Output AC-7-determinism.webm

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

Type "# AC-7: Determinism -- seed=42 produces byte-identical fixture responses across runs"
Enter
Sleep 500ms

Type "cargo test --test ac_7_determinism --features dtu -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
