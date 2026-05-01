Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-003-org-id-mismatch.gif
Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-003-org-id-mismatch.webm

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

Type "# AC-003: cross-org fetch => OrgIdMismatch (not a network error)"
Enter
Sleep 0.4s

Type "grep -n 'OrgIdMismatch\|spec.org_id != self' crates/prism-sensors/src/auth/armis.rs | head -5"
Enter
Sleep 1s

Type "# Test: adapter(org_A).fetch(spec{org_B}) => Err(OrgIdMismatch)"
Enter
Sleep 0.4s

Type "cargo test -p prism-sensors --test org_id_binding test_AC_003 2>&1 | tail -4"
Enter
Sleep 10s

Type "# AC-003 PASS: OrgIdMismatch returned before I/O — 127.0.0.1:1 never contacted"
Enter
Sleep 2s
