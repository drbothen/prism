Output docs/demo-evidence/S-3.0.01/AC-3-fmt-clean.gif
Output docs/demo-evidence/S-3.0.01/AC-3-fmt-clean.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.0.01 && clear"
Enter
Sleep 2s
Show

Type "# AC-3: clean workspace -- lefthook run pre-commit fmt exits 0"
Enter
Sleep 500ms
Type "lefthook run pre-commit --all-files --command fmt && echo exit_code=0 || echo exit_code=non-zero"
Enter
Sleep 8s
Sleep 2s
