Output AC-001-all-8-tests-green.gif
Output AC-001-all-8-tests-green.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.07"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "cargo test -p prism-dtu-jira --features dtu --test org_tagging"
Enter
Sleep 30s
