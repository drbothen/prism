Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-005-downstream-migration.gif
Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-005-downstream-migration.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.06-ImplPhase && clear"
Enter
Sleep 1s
Show

# ── AC-005/AC-006: downstream callers migrated, all tests pass ───────────────
Type "# AC-005/AC-006: 6 test files migrated — all pass, zero E0061 errors"
Enter
Sleep 0.4s

Type "grep -l 'OrgId::from_uuid' crates/prism-sensors/tests/"
Enter
Sleep 1s

Type "# Run full prism-sensors test suite"
Enter
Sleep 0.4s

Type "cargo test -p prism-sensors 2>&1 | tail -5"
Enter
Sleep 30s

Type "# AC-005/AC-006 PASS: 0 E0061 errors; all 6 migrated test files pass"
Enter
Sleep 2s
