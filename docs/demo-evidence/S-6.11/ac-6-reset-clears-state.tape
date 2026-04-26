Output ac-6-reset-clears-state.gif

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

Type "# AC-6: reset() clears received_payloads and rate-limit counter to 0"
Enter
Type "cargo test -p prism-dtu-slack --features dtu --test ac_tests ac_6_reset_clears_received_payloads_and_request_counter ac_6_post_dtu_reset_endpoint_clears_state --nocapture 2>&1 | grep -E 'ac_6|PASSED|FAILED|ok$|error\['"
Enter
Sleep 60s
