Output docs/demo-evidence/S-1.15/AC-013-vp043-wit-rejects-missing.gif
Output docs/demo-evidence/S-1.15/AC-013-vp043-wit-rejects-missing.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-13: VP-043 proptest — WIT validation rejects any component missing required exports, naming them.
# Traces to: BC-2.17.006

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-13: VP-043 proptest — WIT rejects missing exports, names them in error"
Enter
Sleep 500ms
Type "./target/debug/deps/prism_spec_engine-65bdce0b8e3a518e 'proofs::plugin_wit_validation' --nocapture 2>&1"
Enter
Sleep 8s
Sleep 2s
