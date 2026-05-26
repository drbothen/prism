Output AC-006-backwards-compat-no-customers-dir.gif
Output AC-006-backwards-compat-no-customers-dir.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CONFIG-MULTI-TENANT-OVERRIDE-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-006: Backwards compatibility - no customers/ directory (BC-2.06.012 EC-012-001/002)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: absent customers/ dir -> zero overlays, no errors, boot continues"
Enter
Sleep 300ms
Type "# Also: customers/.gitkeep only (no subdirs) -> same zero-overlay result"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_06_012_backcompat_no_customers_dir_uses_type_spec_only)' 2>&1 | tail -8"
Enter
Sleep 10s
