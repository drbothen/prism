Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-001-org-id-signature.gif
Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-001-org-id-signature.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# ── Setup (hidden) ──────────────────────────────────────────────────────────
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.06-ImplPhase && clear"
Enter
Sleep 1s
Show

# ── AC-001: org_id parameter is present (no _org_id prefix) ─────────────────
Type "# AC-001: init_registry_for_org — org_id parameter, no _ prefix"
Enter
Sleep 0.4s

Type "grep -A3 'pub fn init_registry_for_org' crates/prism-sensors/src/lib.rs | head -4"
Enter
Sleep 1s

Type "grep -rn '_org_id' crates/prism-sensors/src/ | wc -l"
Enter
Sleep 1s

Type "# AC-001 PASS: parameter is 'org_id: OrgId' — zero _org_id hits"
Enter
Sleep 2s
