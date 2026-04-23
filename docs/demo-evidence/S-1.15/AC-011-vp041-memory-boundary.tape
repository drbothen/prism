Output docs/demo-evidence/S-1.15/AC-011-vp041-memory-boundary.gif
Output docs/demo-evidence/S-1.15/AC-011-vp041-memory-boundary.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-11: VP-041 proptest — memory limit boundary: at-limit succeeds, over-limit traps (1..=512 MB).
# Traces to: BC-2.17.003

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-11: VP-041 proptest — memory boundary exact across limit_mb in 1..=512"
Enter
Sleep 500ms
Type "./target/debug/deps/prism_spec_engine-65bdce0b8e3a518e 'proofs::plugin_memory::tests::test_BC_2_17_003_vp041_memory_limit_boundary_exact' --nocapture 2>&1"
Enter
Sleep 8s
Sleep 2s
