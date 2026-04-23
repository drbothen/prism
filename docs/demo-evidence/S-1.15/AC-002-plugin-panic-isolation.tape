Output docs/demo-evidence/S-1.15/AC-002-plugin-panic-isolation.gif
Output docs/demo-evidence/S-1.15/AC-002-plugin-panic-isolation.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-2: Plugin trap (unreachable) returns Err(Trapped), host continues.
# Traces to: BC-2.17.001 / INV-PLUGIN-001

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-2: Plugin trap → Err(Trapped), host process unaffected"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_001_ac2_plugin_trap_returns_err_trapped --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
