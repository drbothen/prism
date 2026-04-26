Output ac-11-scan-limit-env-var.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.03-decorators-internal-tables"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-11: scan_limit() env var parsing: default 50000, valid numeric, invalid => default (BC-2.15.011)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_011_scan_limit_default test_BC_2_15_011_scan_limit_valid_numeric test_BC_2_15_011_scan_limit_invalid_string --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
