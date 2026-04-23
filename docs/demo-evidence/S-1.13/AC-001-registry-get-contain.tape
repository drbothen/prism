Output docs/demo-evidence/S-1.13/AC-001-registry-get-contain.gif
Output docs/demo-evidence/S-1.13/AC-001-registry-get-contain.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24
Set Shell zsh

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.13-sensor-write-specs"
Enter
Sleep 2s
Show

Type "# AC-1: registry.get(crowdstrike, contain) -> risk_tier=Irreversible, batch_limit=10"
Enter
Sleep 500ms
Type "cargo test -p prism-spec-engine --test write_endpoint_tests test_BC_2_16_001_write_registry_get_crowdstrike_contain 2>&1 | tail -8"
Enter
Sleep 8s
Sleep 2s
