Output ac-8-auth-reject-mode.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.12-dtu-pagerduty"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-8: auth_mode=reject via /dtu/configure -> HTTP 403 invalid key"
Enter
Type "cargo test -p prism-dtu-pagerduty --features dtu --test fidelity test_ac8_auth_reject_mode_returns_403 --nocapture 2>&1 | grep -E 'ac8|PASSED|FAILED|ok$'"
Enter
Sleep 60s
