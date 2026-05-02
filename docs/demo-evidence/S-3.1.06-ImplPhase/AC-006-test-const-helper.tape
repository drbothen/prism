Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-006-test-const-helper.gif
Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-006-test-const-helper.webm

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

# ── AC-006: OrgId const helper pattern ──────────────────────────────────────
Type "# AC-006: test callers use OrgId::from_uuid(Uuid::from_bytes(sentinel))"
Enter
Sleep 0.4s

Type "grep -n 'cfg(test)\|DEFAULT_ORG_ID_BYTES' crates/prism-sensors/src/lib.rs | head -4"
Enter
Sleep 1s

Type "# Test: sentinel bytes idempotent, equals org_a(), registry has 4 entries"
Enter
Sleep 0.4s

Type "cargo test -p prism-sensors --test org_id_binding test_AC_006 2>&1 | tail -4"
Enter
Sleep 10s

Type "# AC-006 PASS: OrgId sentinel bytes are idempotent and match org_a() helper"
Enter
Sleep 2s
