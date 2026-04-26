Output ac-1-valid-blocks-payload.gif

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

Type "# AC-1: POST /services/{token} with valid Block Kit blocks -> HTTP 200 ok=true + stable message_ts"
Enter
Type "cargo test -p prism-dtu-slack --features dtu --test ac_tests ac_1_valid_blocks_payload_returns_200_ok_with_stable_message_ts ac_1_text_only_payload_returns_200 --nocapture 2>&1 | grep -E 'ac_1|PASSED|FAILED|ok$|error\['"
Enter
Sleep 60s
