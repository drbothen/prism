Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-002-composite-key.gif
Output docs/demo-evidence/S-3.1.06-ImplPhase/AC-002-composite-key.webm

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

Type "# AC-002: AdapterRegistry keyed by (OrgId, SensorType) composite"
Enter
Sleep 0.4s

Type "grep -n 'HashMap' crates/prism-sensors/src/registry.rs"
Enter
Sleep 1s

Type "# Two orgs, same SensorType => distinct Arc pointers"
Enter
Sleep 0.4s

Type "cargo test -p prism-sensors --test org_id_binding test_AC_002 2>&1 | tail -4"
Enter
Sleep 10s

Type "# AC-002 PASS: each org has independent entry; cross-org get returns None"
Enter
Sleep 2s
