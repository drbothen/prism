Output docs/demo-evidence/S-1.15/AC-007-wit-validation-rejection.gif
Output docs/demo-evidence/S-1.15/AC-007-wit-validation-rejection.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-7: Plugin missing required WIT exports → Err(InvalidInterface / E-PLUGIN-001).
# Traces to: BC-2.17.006 / INV-PLUGIN-006

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-7: Invalid WIT → E-PLUGIN-001, plugin not registered"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_006_ac7_invalid_wit_returns_e_plugin_001 --nocapture 2>&1"
Enter
Sleep 3s
Sleep 1s
Type "# Invariant: registry count unchanged after rejection"
Enter
Sleep 300ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_006_ac7_invariant_plugin_not_registered_after_invalid_wit --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
