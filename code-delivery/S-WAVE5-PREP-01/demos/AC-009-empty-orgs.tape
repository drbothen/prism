Output AC-009-empty-orgs.gif
Output AC-009-empty-orgs.webm

Set Width 1200
Set Height 380
Set FontSize 13
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Type "export PATH=/Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01/target/debug:$PATH && cd /Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01"
Enter
Sleep 500ms

Type "# AC-9: zero orgs in config -> exit 2 + canonical error message"
Enter
Sleep 300ms

Type "prism validate-config --config-dir crates/prism-bin/fixtures/config/empty-orgs 2>&1 | grep -v 'at crates'"
Enter
Sleep 1500ms

Type "echo exit=$?"
Enter
Sleep 2s
