Output ac-2-missing-blocks-and-text.gif

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

Type "# AC-2: POST /services/{token} missing both blocks and text -> HTTP 400 invalid_payload"
Enter
Type "cargo test -p prism-dtu-slack --features dtu --test ac_tests ac_2_missing_blocks_and_text_returns_400_invalid_payload --nocapture 2>&1 | grep -E 'ac_2|PASSED|FAILED|ok$|error\['"
Enter
Sleep 60s
