Output docs/demo-evidence/S-1.05/AC-001-crowdstrike-severity-mapping.gif
Output docs/demo-evidence/S-1.05/AC-001-crowdstrike-severity-mapping.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-1 (BC-2.02.003): CrowdStrike detection severity "High" → severity_id=4,
#                       detection_id "ldt:abc123" → finding_info.uid.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-1: CrowdStrike severity_name='High' → severity_id=4; detection_id maps to finding_info.uid"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_003_crowdstrike_severity_high_maps_to_id_4 test_BC_2_02_003_crowdstrike_severity_critical_maps_to_id_5 test_BC_2_02_003_crowdstrike_detection_id_maps_to_finding_info_uid 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
