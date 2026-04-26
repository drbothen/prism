Output docs/demo-evidence/S-2.06/ac-1-fanout-orchestration.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.06-datasource-trait"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1: Fan-out over 6 targets (3 clients x 2 sensors), MAX_FANOUT_CONCURRENCY=10 (BC-2.01.002)"
Enter
Type "cargo test -p prism-sensors --lib -- bc_2_01_002 --nocapture 2>&1 | tail -14"
Enter
Sleep 20s
Sleep 2s
