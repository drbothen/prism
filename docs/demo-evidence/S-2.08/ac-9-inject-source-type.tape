Output docs/demo-evidence/S-2.08/ac-9-inject-source-type.gif

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

Type "# AC-9: inject_source_type — EventStream buffered rows get _source_type=buffered"
Enter
Type "cargo test -p prism-query inject_source_type 2>&1 | grep -E 'test BC_2_08_inject_source_type_(event_stream_buffered|empty|overwrites)|test result.*ok'"
Enter
Sleep 12s
Sleep 2s
