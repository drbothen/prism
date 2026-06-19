Output AC-007-three-tier-cache-rocksdb.gif
Output AC-007-three-tier-cache-rocksdb.webm
Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms
Hide
Sleep 2s
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.14-REDO && clear"
Enter
Sleep 1s
Show
Type "# AC-7: 3-tier cache — Tier 3 RocksDB persists without calling live source again (BC-2.19.002)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_ac_7_csv_source_asset_owner_spec_loads_correctly)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Integration: Tier 3 RocksDB wiring in production query engine"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-bin -E 'test(test_infusion_tier3_production_read_without_source)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 5s
Sleep 2s
