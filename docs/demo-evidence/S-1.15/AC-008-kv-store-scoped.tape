Output docs/demo-evidence/S-1.15/AC-008-kv-store-scoped.gif
Output docs/demo-evidence/S-1.15/AC-008-kv-store-scoped.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-8: KV store scoped per plugin — plugin B cannot see plugin A's keys.
# Traces to: BC-2.17.002 / EC-17-008

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-8: KV store isolation — plugin B gets None for plugin A's key"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_002_ac8_kv_store_scoped_per_plugin --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
