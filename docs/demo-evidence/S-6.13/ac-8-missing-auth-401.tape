Output ac-8-missing-auth-401.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.13-dtu-jira"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-8: Request without Authorization: Basic -> 401 'Basic authentication required'"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_missing_auth_returns_401 test_bearer_scheme_returns_401 test_invalid_base64_in_basic_auth_returns_401 --nocapture 2>&1 | grep -E 'test_missing_auth|test_bearer|test_invalid_base64|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
