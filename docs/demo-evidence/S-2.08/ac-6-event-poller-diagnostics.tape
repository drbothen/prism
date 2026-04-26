Output docs/demo-evidence/S-2.08/ac-6-event-poller-diagnostics.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.08-event-tables"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-8 (diagnostics): PollerDiagnostics struct — cold_start, last_poll_time, record count"
Enter
Type "cargo test -p prism-sensors poller 2>&1 | grep -E 'test BC_2_08_poller_diagnostics|test result.*ok'"
Enter
Sleep 15s
Sleep 2s
