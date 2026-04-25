Output ac-4-scan-prefix.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.01-rocksdb-init"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-4: scan returns only prefix-matching keys"
Enter
Type "cargo test -p prism-storage --test integration test_ac_4_scan_prefix_returns_only_matching -- --nocapture --test-threads=1 2>&1 | tail -8"
Enter
Sleep 10s
