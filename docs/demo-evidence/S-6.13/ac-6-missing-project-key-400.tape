Output ac-6-missing-project-key-400.gif

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

Type "# AC-6: POST issue missing project.key -> 400 {errors: {project: required}}"
Enter
Type "cargo test -p prism-dtu-jira --features dtu --test fidelity test_missing_project_key_returns_400 --nocapture 2>&1 | grep -E 'test_missing_project|PASSED|FAILED|ok$|error\['"
Enter
Sleep 90s
