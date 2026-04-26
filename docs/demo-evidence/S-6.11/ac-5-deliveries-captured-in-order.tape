Output ac-5-deliveries-captured-in-order.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.11-dtu-slack"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-5: 3 webhook deliveries captured in order via GET /dtu/received-payloads"
Enter
Type "cargo test -p prism-dtu-slack --features dtu --test ac_tests ac_5_three_deliveries_captured_in_order ac_5_in_process_received_payloads_api_matches_http_endpoint --nocapture 2>&1 | grep -E 'ac_5|PASSED|FAILED|ok$|error\['"
Enter
Sleep 60s
