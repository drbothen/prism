Output docs/demo-evidence/S-1.13/AC-004-write-table-descriptor.gif
Output docs/demo-evidence/S-1.13/AC-004-write-table-descriptor.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24
Set Shell zsh

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.13-sensor-write-specs"
Enter
Sleep 2s
Show

Type "# AC-4: WriteTableDescriptor -> sql_table=crowdstrike_contained_hosts, write_only=true"
Enter
Sleep 500ms
Type "cargo test -p prism-spec-engine --test write_endpoint_tests write_table_descriptor 2>&1 | tail -8"
Enter
Sleep 8s
Sleep 2s
