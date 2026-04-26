Output docs/demo-evidence/S-2.07/ac-1-crowdstrike-oauth2.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.07-per-sensor-auth"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1 & AC-2: CrowdStrike OAuth2 token cache + 401 retry (BC-2.01.005)"
Enter
Type "cargo test -p prism-sensors --test test_crowdstrike 2>&1 | tail -12"
Enter
Sleep 8s
Sleep 2s
