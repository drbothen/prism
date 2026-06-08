Output docs/demo-evidence/S-DEMO-003/AC-002-boot-probe-tier3a.gif
Output docs/demo-evidence/S-DEMO-003/AC-002-boot-probe-tier3a.webm

Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-002: prism start boot probe uses Tier-3a OrgId-keyed path (TV-BOOT-P-001)
# Closes F-P14-CRIT-001
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-002: boot step-5 Tier-3a OrgId-keyed probe (F-P14-CRIT-001 fix)"
Enter
Sleep 500ms

Type "# TV-BOOT-P-001: KeyringCredentialProbe uses OrgId-keyed get_by_org"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin -E 'test(boot_probe_tier3a)'"
Enter
Sleep 10s
