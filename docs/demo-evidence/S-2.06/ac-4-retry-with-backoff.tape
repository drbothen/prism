Output docs/demo-evidence/S-2.06/ac-4-retry-with-backoff.gif

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

Type "# AC-4: Exponential backoff 2s base, jitter, non-transient 400/404 not retried (BC-2.01.014)"
Enter
Type "cargo test -p prism-sensors --lib -- bc_2_01_014 --nocapture 2>&1 | tail -27"
Enter
Sleep 20s
Sleep 2s
