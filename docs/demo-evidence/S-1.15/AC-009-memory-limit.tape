Output docs/demo-evidence/S-1.15/AC-009-memory-limit.gif
Output docs/demo-evidence/S-1.15/AC-009-memory-limit.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-9: Memory limit enforced — over-64MB returns Err(MemoryExceeded).
# AC-9 error path + at-limit success path.
# Traces to: BC-2.17.003 / INV-PLUGIN-003

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-9 (error path): allocation > 64MB → Err(MemoryExceeded { limit_mb: 64 })"
Enter
Sleep 500ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_003_ac9_memory_limit_exceeded_returns_err --nocapture 2>&1"
Enter
Sleep 3s
Sleep 1s
Type "# AC-9 (success path): allocation exactly at 64MB → Ok"
Enter
Sleep 300ms
Type "./target/debug/deps/plugin_tests-4f46a65c03c2d0f0 test_BC_2_17_003_ec17_009_at_limit_allocation_succeeds --nocapture 2>&1"
Enter
Sleep 3s
Sleep 2s
