Output ac-7-storage-lock-held.gif

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

Type "# AC-7: StorageLockHeld error when DB locked by another process"
Enter
Type "cargo test -p prism-storage --test integration test_ac_7_storage_lock_held_error -- --nocapture --test-threads=1 2>&1 | tail -8"
Enter
Sleep 10s
