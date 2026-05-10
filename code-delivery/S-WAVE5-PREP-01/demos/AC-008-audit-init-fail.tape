Output AC-008-audit-init-fail.gif
Output AC-008-audit-init-fail.webm

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

Type "# AC-8: audit subsystem init failure -> exit 4"
Enter
Sleep 300ms

Type "PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure prism validate-config --config-dir crates/prism-bin/fixtures/config/valid 2>&1 | grep -v 'at crates'"
Enter
Sleep 1500ms

Type "echo exit=$?"
Enter
Sleep 2s
