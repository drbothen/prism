# VHS Demo — AC-016 + AC-017 + AC-018: query column resolution over OCSF surface
# AC-016: E-QUERY-038 gate rejects unavailable columns using OCSF-flattened names
# AC-017: SELECT with flattened OCSF names succeeds (finding_info_uid, metadata_uid)
# AC-018: available_columns in E-QUERY-038 message lists OCSF-flattened names
# Tests: RG-Q-001..RG-Q-009 (ocsf_column_routing suite)

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-016-018-query-column-resolution.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-016-018-query-column-resolution.webm

Require cargo

Set Shell "zsh"
Set WaitTimeout 60s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 17
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

# Full ocsf_column_routing test suite: RG-Q-001..RG-Q-009
Type "cargo nextest run -p prism-query -E 'test(ocsf_column_routing)' 2>&1 | tail -10"
Enter
Sleep 20s
Sleep 2s
