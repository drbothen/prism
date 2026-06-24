# AC-8: probe_table naming missing table -> E-SPEC-026 error from SpecLoader::parse() (BC-2.16.009 Rule 8)
# Success path: valid probe_table passes Rule 8 (all 4 bundled specs pass).

Output docs/demo-evidence/S-5.04/AC-08-espec026-invalid-probe-table.gif
Output docs/demo-evidence/S-5.04/AC-08-espec026-invalid-probe-table.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-5.04"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "cargo nextest run -p prism-spec-engine -E 'test(probe_table_names_missing_table_returns_ESpec026) + test(validates_all_4_bundled_specs)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 60s

Sleep 2s
