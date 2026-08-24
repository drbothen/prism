# VHS Demo — AC-011 + AC-013 (error paths)
# AC-011: unknown ocsf_class emits WARN and Ok(0) (not Err)
# AC-013: validate_ocsf_column_collisions detects [§J1] shadow, [§J2] reserved name, [§J4] intra-table dup
#         — all return E-SPEC-030
# Tests: RG-009, RG-010, RG-Q-012, RG-Q-013, RG-Q-014

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-011-013-error-paths.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-011-013-error-paths.webm

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

# Unknown ocsf_class -> WARN + class_uid=0 Ok (AC-011)
# ocsf_field flattens to reserved name -> Err (AC-013 §J2)
# Shadow collision -> Err (AC-013 §J1)
# Intra-table collision -> Err (AC-013 §J4)
Type "cargo nextest run -p prism-bin -E 'test(record_batch_ocsf)' 2>&1 | tail -10"
Enter
Sleep 20s
Sleep 2s
