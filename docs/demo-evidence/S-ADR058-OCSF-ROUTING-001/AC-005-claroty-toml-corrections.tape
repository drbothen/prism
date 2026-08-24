# VHS Demo — AC-005: claroty.sensor.toml KF-01..KF-12 + ocsf_column_naming = true
# Shows the TOML corrections (ocsf_class, ocsf_field values per OCSF v1.7.0) and flag

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-005-claroty-toml-corrections.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-005-claroty-toml-corrections.webm

Require grep

Set Shell "zsh"
Set WaitTimeout 60s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-ADR058-OCSF-ROUTING-001"
Enter
Sleep 1.5s
Type "clear"
Enter
Sleep 0.5s
Show

# Show ocsf_column_naming = true flag
Type "grep 'ocsf_column_naming' crates/prism-sensors/specs/claroty.sensor.toml"
Enter
Sleep 3s

Type "clear"
Enter
Sleep 0.5s

# Show all ocsf_class values (entity_management, inventory_info per KF-01..KF-04)
Type "grep 'ocsf_class' crates/prism-sensors/specs/claroty.sensor.toml"
Enter
Sleep 3s

Type "clear"
Enter
Sleep 0.5s

# Show ocsf_field corrections (subset: finding_info.uid, metadata.uid, device.type_category)
Type "grep -c 'ocsf_field' crates/prism-sensors/specs/claroty.sensor.toml"
Enter
Sleep 3s
Sleep 2s
