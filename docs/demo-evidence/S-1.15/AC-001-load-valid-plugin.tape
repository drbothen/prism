Output docs/demo-evidence/S-1.15/AC-001-load-valid-plugin.gif
Output docs/demo-evidence/S-1.15/AC-001-load-valid-plugin.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-1: Valid .prx plugin loads and registers in the runtime.
# Traces to: BC-2.17.006 / INV-PLUGIN-006

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-1: Load valid .prx infusion plugin into PluginRuntime"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_006_ac1_load_valid_infusion_plugin --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
