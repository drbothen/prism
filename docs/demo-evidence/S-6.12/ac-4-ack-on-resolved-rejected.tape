Output ac-4-ack-on-resolved-rejected.gif

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

Type "# AC-4: acknowledge on Resolved incident -> HTTP 400"
Enter
Type "cargo test -p prism-dtu-pagerduty --features dtu --test fidelity test_ac4_ack_on_resolved_returns_400 --nocapture 2>&1 | grep -E 'ac4|PASSED|FAILED|ok$'"
Enter
Sleep 60s
