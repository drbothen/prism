Output docs/demo-evidence/S-DEMO-003/AC-009-010-tier3-e2e-resolution.gif
Output docs/demo-evidence/S-DEMO-003/AC-009-010-tier3-e2e-resolution.webm

Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-009 (CRIT-1): credential written by set_by_org → resolved at Tier-3 (RG-034-001)
# AC-010 (CRIT-2): OrgId-keyed write == OrgId-keyed read, slug-keyed write invisible
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-009 (CRIT-1) + AC-010 (CRIT-2): Tier-3 end-to-end namespace"
Enter
Sleep 500ms

Type "# RG-034-001: credential written via set_by_org is resolved at Tier-3"
Enter
Sleep 400ms

Type "# RG-034-004: OrgId-keyed key exists; slug-keyed key does NOT"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-credentials --features test-helpers -E 'test(tier3_credential_written)'"
Enter
Sleep 10s

Type "cargo nextest run -p prism-bin -E 'test(writes_org_id_keyed_namespace)'"
Enter
Sleep 8s
