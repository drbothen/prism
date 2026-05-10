Output AC-004-invalid-toml.gif
Output AC-004-invalid-toml.webm

Set Width 1200
Set Height 450
Set FontSize 13
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Type "export PATH=/Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01/target/debug:$PATH && cd /Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01"
Enter
Sleep 500ms

Type "# AC-4: invalid TOML -> exit 2 with line/field context in stderr"
Enter
Sleep 300ms

Type "cat crates/prism-bin/fixtures/config/invalid-toml/prism.toml"
Enter
Sleep 800ms

Type "prism validate-config --config-dir crates/prism-bin/fixtures/config/invalid-toml 2>&1 | grep -v 'at crates'"
Enter
Sleep 1500ms

Type "echo exit=$?"
Enter
Sleep 2s
