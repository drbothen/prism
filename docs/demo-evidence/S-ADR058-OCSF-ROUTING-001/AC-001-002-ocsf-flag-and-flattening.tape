# VHS Demo — AC-001 + AC-002
# AC-001: SensorSpec gains ocsf_column_naming field with serde(default)
# AC-002: ocsf_field_to_arrow_name helper correctly flattens dotted OCSF paths
# Tests: RG-001, RG-002, RG-003, RG-004

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-001-002-ocsf-flag-and-flattening.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-001-002-ocsf-flag-and-flattening.webm

Require cargo

Set Shell "zsh"
Set WaitTimeout 60s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
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

# AC-001: ocsf_column_naming field defaults false, parses true from TOML
# AC-002: ocsf_field_to_arrow_name replaces dots with underscores
Type "cargo nextest run -p prism-spec-engine -E 'test(ocsf_column_naming) | test(ocsf_field_to_arrow_name)' 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
