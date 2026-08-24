# VHS Demo — AC-010: KF-01..KF-12 wire-shape corrections
# finding_info.* fields (KF-03,04,12), reserved→raw_extensions (KF-08,09,10),
# audit_logs category (KF-11), finding_info_uid (KF-07), metadata_uid (OQ-005), device_type_label (KF-06)
# Tests: RG-014..RG-022, RG-024

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-010-wire-shape-kf-corrections.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-010-wire-shape-kf-corrections.webm

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

# KF wire-shape tests: finding_info.*, reserved→raw_extensions, kf11 category, metadata_uid, device_type_label
Type "cargo nextest run -p prism-bin -E 'test(claroty_alerts_finding_info) | test(device_type_label) | test(reserved_fields_go_to_raw) | test(kf11_category) | test(metadata_uid_top_level) | test(finding_info_uid_wire_shape)' 2>&1 | tail -10"
Enter
Sleep 20s
Sleep 2s
