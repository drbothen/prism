Output docs/demo-evidence/S-2.06/ac-5-dual-semaphore.gif

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

Type "# AC-5: HTTP semaphore 200 permits, 201st task blocks not rejected (BC-2.01.014/EC-003)"
Enter
Type "cargo test -p prism-sensors --lib -- bc_2_01_http_semaphore --nocapture 2>&1 | tail -14"
Enter
Sleep 20s
Sleep 2s
