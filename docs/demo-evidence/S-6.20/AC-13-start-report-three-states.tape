# AC-13: StartReport three-state enum -- all-success, abort, and partial-success outputs.
#
# Drives the AC-13 integration tests which exercise all three StartReport shapes:
#   - All 6 clones start: successfully_started.len()==6, rest empty
#   - Abort (continue_on_error=false, bind fail): failed_at.is_some(), cleaned_up non-empty
#   - Partial success (continue_on_error=true, bind fail): skipped_due_to_error non-empty

Output AC-13-start-report-three-states.gif
Output AC-13-start-report-three-states.webm

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

Type "# AC-13: StartReport three-state: all-success | abort | partial-success"
Enter
Sleep 500ms

Type "cargo test --test ac_13_start_report_semantics --features dtu -- --nocapture 2>&1 | tail -25"
Enter
Wait+Screen /test result/
Sleep 2s
