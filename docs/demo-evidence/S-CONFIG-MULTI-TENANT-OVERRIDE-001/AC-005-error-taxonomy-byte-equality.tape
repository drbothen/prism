Output AC-005-error-taxonomy-byte-equality.gif
Output AC-005-error-taxonomy-byte-equality.webm

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

Type "# AC-005: Error taxonomy and SpecErrorCode byte-equality safety net (BC-2.06.016)"
Enter
Sleep 500ms

Type "# POL-25: test reads error-taxonomy.md at runtime and byte-compares production"
Enter
Sleep 300ms
Type "# SpecError::message against canonical template for E-SPEC-019..023."
Enter
Sleep 300ms
Type "# Also: 4-test multi-error aggregation suite (EC-016-001, 002, 003)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_06_016)' 2>&1 | tail -12"
Enter
Sleep 10s
