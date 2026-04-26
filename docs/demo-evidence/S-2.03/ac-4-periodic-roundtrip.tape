Output ac-4-periodic-roundtrip.gif

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

Type "# AC-4: store_periodic/load_periodic round-trip via bincode+RocksDB; fresh tenant=None (BC-2.15.010)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_010_store_and_load_periodic_round_trip test_BC_2_15_010_ec001_load_periodic_fresh_tenant_returns_none --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
