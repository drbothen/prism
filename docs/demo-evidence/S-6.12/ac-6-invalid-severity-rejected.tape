Output ac-6-invalid-severity-rejected.gif

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

Type "# AC-6: severity 'fatal' (invalid) -> HTTP 400 invalid severity"
Enter
Type "cargo test -p prism-dtu-pagerduty --features dtu --test fidelity test_ac6_invalid_severity_returns_400 --nocapture 2>&1 | grep -E 'ac6|PASSED|FAILED|ok$'"
Enter
Sleep 60s
