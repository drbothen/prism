Output docs/demo-evidence/S-1.15/AC-006-hot-reload.gif
Output docs/demo-evidence/S-1.15/AC-006-hot-reload.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-6: Hot reload atomic swap — arc-swap, in-flight calls complete.
# Traces to: BC-2.17.005 / INV-PLUGIN-005

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-6: Hot reload — atomic arc-swap, old Arc stays alive for in-flight calls"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_005_ac6_hot_reload_atomic_swap --nocapture 2>&1"
Enter
Sleep 3s
Sleep 1s
Type "# Error path: failed reload retains old plugin"
Enter
Sleep 300ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_005_ec17_005_failed_recompile_retains_old_plugin --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
