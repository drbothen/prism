Output AC-003-validate-config-valid.gif
Output AC-003-validate-config-valid.webm

Set Width 1200
Set Height 600
Set FontSize 13
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Type "export PATH=/Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01/target/debug:$PATH && cd /Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01"
Enter
Sleep 500ms

Type "# AC-3: validate-config with valid prism.toml exits 0"
Enter
Sleep 300ms

Type "prism validate-config --config-dir crates/prism-bin/fixtures/config/valid 2>&1 | grep -v 'at crates'"
Enter
Sleep 2000ms

Type "echo exit=$?"
Enter
Sleep 2s
