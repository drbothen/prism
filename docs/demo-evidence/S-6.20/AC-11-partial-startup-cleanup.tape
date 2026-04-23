# AC-11: Partial startup failure -- bind collision on one clone triggers abort and cleanup.
#
# Drives the AC-11 integration tests which pre-bind a port, configure armis to use it,
# start all clones, and assert: Err returned, cleaned_up_after_failure lists 3 clones,
# failed_at is ("armis", AddrInUse), no port leak.

Output AC-11-partial-startup-cleanup.gif
Output AC-11-partial-startup-cleanup.webm

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

Type "# AC-11: Bind collision triggers abort + cleanup of already-started clones"
Enter
Sleep 500ms

Type "cargo test --test ac_11_partial_startup_cleanup --features dtu -- --nocapture 2>&1 | tail -25"
Enter
Wait+Screen /test result/
Sleep 2s
