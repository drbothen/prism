# VHS Demo — AC-019 + AC-020: zero-col OCSF table + projection invariant
# AC-019: zero-col table (no ocsf_field) registers class_uid + _sensor; A+W sub-case adds raw_extensions + WARN
# AC-020: ocsf_projected_column_names is the single authoritative impl; registry/engine/describe agree
# Tests: RG-Q-010, RG-Q-011, RG-Q-015, RG-Q-016

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-019-020-zero-col-and-projection-invariant.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-019-020-zero-col-and-projection-invariant.webm

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

# zero-col OCSF (accepts class_uid+_sensor, rejects raw col.name), A+W sub-case (raw_extensions + WARN), projection invariant
Type "cargo nextest run -p prism-query -E 'test(zero_col_ocsf_table) | test(zero_tier1_with_tier2) | test(ocsf_projected_names_all)' 2>&1 | tail -10"
Enter
Sleep 20s
Sleep 2s
