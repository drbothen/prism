# AC-005: Regression safety — all 51 prior tests still pass
#
# Runs the full logical_isolation_test and network_isolation_test suites to confirm
# that S-3.3.05 builder ergonomics changes do not regress any of the 34 logical +
# 16 network + 1 timeout tests introduced in S-3.3.03 and S-3.3.04.
# Traces to: BC-3.5.001, BC-3.5.002 — all prior invariants preserved

Output AC-005-regression-safe.gif
Output AC-005-regression-safe.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 180s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.05"
  Enter
  Sleep 500ms
Show

Type "# AC-005: Regression safety — 51 prior tests still pass after S-3.3.05"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test --test network_isolation_test 2>&1 | tail -30"
Enter

Wait+Screen /test result/
Sleep 2s
