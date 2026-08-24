# VHS Demo — AC-009: class_selector wiring for Claroty tables
# entity_management -> class_uid=3004, inventory_info -> class_uid=5001
# Wire-shape: audit_logs rows carry class_uid=3004, devices rows carry class_uid=5001
# Tests: RG-011, RG-012, RG-013, RG-022, RG-023

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-009-class-selector-entity-management.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-009-class-selector-entity-management.webm

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

# class_selector arms: entity_management=3004, inventory_info=5001 (prism-ocsf)
# wire-shape: audit_logs class_uid=3004, devices class_uid=5001 (prism-bin)
Type "cargo nextest run -p prism-ocsf -p prism-bin -E 'test(class_selector) | test(class_uid_is_)' 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
