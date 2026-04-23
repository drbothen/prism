Output docs/demo-evidence/S-1.15/AC-003-cpu-timeout.gif
Output docs/demo-evidence/S-1.15/AC-003-cpu-timeout.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-3: Infinite loop plugin returns Err(Timeout) within 6s.
# Traces to: BC-2.17.004 / INV-PLUGIN-004

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-3: Infinite loop → Err(Timeout) within 5s limit + 1s tolerance"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_004_ac3_infinite_loop_returns_err_timeout --nocapture 2>&1"
Enter
Sleep 8s
Sleep 2s
