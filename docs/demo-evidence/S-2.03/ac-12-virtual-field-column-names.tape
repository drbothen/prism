Output ac-12-virtual-field-column-names.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.03-decorators-internal-tables"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-12: VirtualField::Sensor=_sensor, Client=_client, SourceTable=_source_table (BC-2.15.009)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_009_virtual_field_column_names --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
