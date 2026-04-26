Output docs/demo-evidence/S-2.08/ac-3-event-buffer-write-scan.gif

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

Type "# AC-2 (buffer side): EventBufferStore write + scan operations"
Enter
Type "cargo test -p prism-sensors event_buffer 2>&1 | grep -E 'test BC_2_08_(write_events|scan_events|has_data|buffer_size)|test result.*ok'"
Enter
Sleep 15s
Sleep 2s
