Output docs/demo-evidence/S-6.08/EC-all-edge-cases.gif
Output docs/demo-evidence/S-6.08/EC-all-edge-cases.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# EC-001..EC-006: all 6 edge cases (unrecognized filter, 404 on missing tag,
# unknown group_by, pagination beyond last page, 422 simulation, latency layer)

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test edge_cases"
Enter
Sleep 15s
