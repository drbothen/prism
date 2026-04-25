Output ac-1-open-16-cfs.gif

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

Type "# AC-1: Fresh open initializes all 16 column families"
Enter
Type "cargo test -p prism-storage --test integration test_ac_1_open_initializes_all_16_column_families -- --nocapture --test-threads=1 2>&1 | tail -8"
Enter
Sleep 10s
