Output ac-5-set-dirty-sync.gif

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

Type "# AC-5: set_dirty writes with sync semantics to dirty_bits CF"
Enter
Type "cargo test -p prism-storage --test integration test_ac_5_set_dirty_writes_with_sync -- --nocapture --test-threads=1 2>&1 | tail -8"
Enter
Sleep 10s
