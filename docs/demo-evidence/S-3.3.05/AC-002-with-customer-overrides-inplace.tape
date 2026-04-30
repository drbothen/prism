# AC-002: with_customer_overrides applies closure in-place to existing spec
#
# Runs the deduplication test: with_customer + with_customer_overrides for the same
# slug must mutate the existing CustomerSpec, not insert a duplicate. Verified via
# endpoint count (1 endpoint expected; 5 indicates duplicate insertion).
# Traces to: BC-3.5.001 precondition 2, S-3.3.05 Task 2

Output AC-002-with-customer-overrides-inplace.gif
Output AC-002-with-customer-overrides-inplace.webm

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
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.05"
  Enter
  Sleep 500ms
Show

Type "# AC-002: with_customer_overrides applies in-place (slug deduplication)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test builder_ergonomics_test with_customer -- --nocapture 2>&1 | tail -30"
Enter

Wait+Screen /test result/
Sleep 2s
