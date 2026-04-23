Output docs/demo-evidence/S-1.08/AC-002-runtime-allow.gif
Output docs/demo-evidence/S-1.08/AC-002-runtime-allow.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-2 (BC-2.04.002): Runtime per-client TOML config grants Allow for read capability

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-2: Runtime TOML config — check_permission returns Allow for enabled read path"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_BC_2_04_002 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
