Output ac-3-unknown-top-level-field.gif

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

Type "# AC-3: POST /services/{token} with unknown top-level field -> HTTP 400 unknown_field"
Enter
Type "cargo test -p prism-dtu-slack --features dtu --test ac_tests ac_3_unknown_top_level_field_returns_400_unknown_field ac_3_all_allowed_top_level_fields_are_accepted --nocapture 2>&1 | grep -E 'ac_3|PASSED|FAILED|ok$|error\['"
Enter
Sleep 60s
