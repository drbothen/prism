# AC-12: Continue-on-error -- same bind collision but continue_on_error=true keeps others running.
#
# Drives the AC-12 integration tests which pre-bind a port, configure armis with
# continue_on_error=true, start all clones, and assert: Ok returned, armis in
# skipped_due_to_error, remaining 5 clones in successfully_started.

Output AC-12-continue-on-error.gif
Output AC-12-continue-on-error.webm

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

Type "# AC-12: continue_on_error=true skips failed clone, keeps others running"
Enter
Sleep 500ms

Type "cargo test --test ac_12_continue_on_error --features dtu -- --nocapture 2>&1 | tail -25"
Enter
Wait+Screen /test result/
Sleep 2s
