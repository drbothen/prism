Output docs/demo-evidence/S-1.15/AC-004-sandbox-no-filesystem.gif
Output docs/demo-evidence/S-1.15/AC-004-sandbox-no-filesystem.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-4: WASI filesystem call rejected at load time — no WASI linked.
# Traces to: BC-2.17.002 / INV-PLUGIN-002

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-4: WASI-importing plugin rejected — no filesystem access"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_002_ac4_wasi_filesystem_not_accessible --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
