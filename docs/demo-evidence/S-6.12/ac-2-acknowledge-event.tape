Output ac-2-acknowledge-event.gif

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

Type "# AC-2: POST /v2/enqueue acknowledge -> HTTP 200 + status Acknowledged"
Enter
Type "cargo test -p prism-dtu-pagerduty --features dtu --test fidelity test_full_lifecycle_trigger_ack_resolve --nocapture 2>&1 | grep -E 'AC-2|lifecycle|PASSED|FAILED|ok$'"
Enter
Sleep 60s
