Output ac-1-trigger-event.gif

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

Type "# AC-1: POST /v2/enqueue trigger -> HTTP 202 + status Triggered"
Enter
Type "cargo test -p prism-dtu-pagerduty --features dtu --test fidelity test_full_lifecycle_trigger_ack_resolve --nocapture 2>&1 | grep -E 'test_full_lifecycle|PASSED|FAILED|ok$|error\['"
Enter
Sleep 60s
