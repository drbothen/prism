Output ac-5-trigger-idempotent.gif

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

Type "# AC-5: re-trigger on active incident -> HTTP 202 idempotent (no duplicate)"
Enter
Type "cargo test -p prism-dtu-pagerduty --features dtu --test fidelity test_ac5_trigger_idempotent_on_active_incident --nocapture 2>&1 | grep -E 'ac5|PASSED|FAILED|ok$'"
Enter
Sleep 60s
