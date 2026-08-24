# VHS Demo — AC-021: §J4 spec-load collision rejection with E-SPEC-030
# Four collision categories at spec-load:
#   [§J1] Tier-1 vs Tier-1 shadow (two ocsf_fields flatten to same Arrow name)
#   [§J2] reserved synthesized name (class_uid, category_uid, _sensor, raw_extensions)
#   [§J4] intra-table duplicate flattened name
#   [§J1] Tier-1 vs Tier-2 shadow (ocsf_field flattens to same name as a non-ocsf col.name)
# Tests: RG-Q-012, RG-Q-013, RG-Q-014, RG-Q-016 (collision subcases)

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-021-spec-load-collision-rejection.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-021-spec-load-collision-rejection.webm

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

# All four collision categories produce E-SPEC-030 at spec-load (BC_2_16_003)
Type "cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_003_ocsf_collision)' 2>&1 | tail -10"
Enter
Sleep 20s
Sleep 2s
