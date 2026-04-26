Output docs/demo-evidence/S-2.06/ac-2-partial-result-classification.gif

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

Type "# AC-2: 5/6 succeed, 1 returns 503 -> successes=5, errors=1, is_transient=true (BC-2.01.010)"
Enter
Type "cargo test -p prism-sensors --lib -- bc_2_01_010 --nocapture 2>&1 | tail -12"
Enter
Sleep 20s
Sleep 2s
