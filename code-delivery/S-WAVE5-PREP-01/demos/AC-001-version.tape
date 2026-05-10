Output AC-001-version.gif
Output AC-001-version.webm

Set Width 1200
Set Height 400
Set FontSize 14
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Type "export PATH=/Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01/target/debug:$PATH"
Enter
Sleep 500ms

Type "# AC-1 + AC-2: prism --version and version subcommand both exit 0"
Enter
Sleep 300ms

Type "prism --version"
Enter
Sleep 1s

Type "echo exit=$?"
Enter
Sleep 500ms

Type "prism version 2>&1 | grep -v 'at crates'"
Enter
Sleep 1500ms

Type "echo exit=$?"
Enter
Sleep 2s
