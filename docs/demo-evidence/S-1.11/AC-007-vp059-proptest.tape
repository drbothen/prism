# AC-007 / VP-059: All-errors-collected property
Output docs/demo-evidence/S-1.11/AC-007-vp059-proptest.gif
Output docs/demo-evidence/S-1.11/AC-007-vp059-proptest.webm

Set Shell "bash"
Set FontSize 13
Set Width 1200
Set Height 600
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Padding 16
Set PlaybackSpeed 1.0

Type "/Users/jmagady/dev/prism/.worktrees/S-1.11-spec-loading/target/debug/examples/demo_spec_loading vp059"
Enter
Sleep 4s
Type "cargo test -p prism-spec-engine test_BC_2_16_009_invariant_all_errors_collected 2>&1 | tail -3"
Enter
Sleep 8s
