# AC-002: claroty_audit_logs — timestamp-only sort_by alongside filter_by
# Traces: BC-2.16.013 postcondition 1 sort-by + EC-016-013-011
# Live-data-free: reads only claroty.sensor.toml (no network, no tenant data)
# Note: timestamp-only canonical — compound form with id RETIRED (live-validated)

Output AC-002-audit_logs-sort_by.gif
Output AC-002-audit_logs-sort_by.webm

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Set WaitTimeout 30s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/DEFECT-CLAROTY-SORTBY-DETERMINISM-001"
Enter
Sleep 0.5s
Type "clear"
Enter
Sleep 0.3s
Show

Type "python3 docs/demo-evidence/DEFECT-CLAROTY-SORTBY-DETERMINISM-001/show_sort_by.py audit_logs"
Enter
Wait+Screen /RETIRED/
Sleep 1s

Type "python3 -u docs/demo-evidence/DEFECT-CLAROTY-SORTBY-DETERMINISM-001/demo_error_path.py audit_logs"
Enter
Wait+Screen /DEFECT/
Sleep 1s

Sleep 2s
