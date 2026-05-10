Output AC-012-panic-hook.gif
Output AC-012-panic-hook.webm

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

Type "# AC-12: injected panic -> tracing::error! log + exit 1"
Enter
Sleep 300ms

Type "PRISM_TEST_INJECT_PANIC=true prism validate-config --config-dir crates/prism-bin/fixtures/config/valid 2>&1 | grep -v 'at crates'"
Enter
Sleep 1500ms

Type "echo exit=$?"
Enter
Sleep 2s
