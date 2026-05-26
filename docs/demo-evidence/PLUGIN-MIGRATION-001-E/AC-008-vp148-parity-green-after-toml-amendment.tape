# PLUGIN-MIGRATION-001-E / AC-008
# VP-148 Parity Test Remains GREEN After TOML Amendment
# Traces to: BC-2.16.013 INV-PARITY-001, VP-148 replacement-before-deletion enforcement
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-008-vp148-parity-green-after-toml-amendment.gif
Output AC-008-vp148-parity-green-after-toml-amendment.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/PLUGIN-MIGRATION-001-E"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 1s
Show

Type "# AC-008: VP-148 DTU parity GREEN — TOML amendment preserves byte-identical OCSF output"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_008)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
