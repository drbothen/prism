Output docs/demo-evidence/S-1.15/AC-012-vp042-hot-reload-retains.gif
Output docs/demo-evidence/S-1.15/AC-012-vp042-hot-reload-retains.webm

Require bash

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 480
Set Theme "Dracula"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

# AC-12: VP-042 proptest — failed hot reload retains old Arc<LoadedPlugin>.
# Traces to: BC-2.17.005

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.15-wasm-runtime && clear"
Enter
Sleep 500ms
Show

Type "# AC-12: VP-042 proptest — failed reload never corrupts registry entry"
Enter
Sleep 500ms
Type "./target/debug/deps/prism_spec_engine-65bdce0b8e3a518e 'proofs::plugin_hot_reload' --nocapture 2>&1"
Enter
Sleep 8s
Sleep 2s
