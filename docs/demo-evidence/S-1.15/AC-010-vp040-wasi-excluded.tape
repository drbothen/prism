Output docs/demo-evidence/S-1.15/AC-010-vp040-wasi-excluded.gif
Output docs/demo-evidence/S-1.15/AC-010-vp040-wasi-excluded.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-10: VP-040 — Linker excludes all WASI namespace imports.
# Proptest fallback: WASI-importing component rejected at instantiate_pre.
# Traces to: BC-2.17.002

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-10: VP-040 proptest — no wasi: imports in plugin Linker"
Enter
Sleep 500ms
Type "./target/debug/deps/prism_spec_engine-65bdce0b8e3a518e 'proofs::plugin_linker::tests::test_BC_2_17_002_vp040_wasi_importing_component_rejected' --nocapture 2>&1"
Enter
Sleep 4s
Sleep 2s
