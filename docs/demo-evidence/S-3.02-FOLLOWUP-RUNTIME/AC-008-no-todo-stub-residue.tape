# AC-8 — No todo!() or unimplemented!() remains in the 9 implementation sites
# Traces to POL-12
# Story: S-3.02-FOLLOWUP-RUNTIME
#
# Success path: zero matches in engine.rs, materialization.rs, internal_tables.rs
# Error path contrast: boot.rs steps 9/10 (out-of-scope) DO have todo!()

Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/AC-008-no-todo-stub-residue.gif
Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/AC-008-no-todo-stub-residue.webm

Require grep

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.02-FOLLOWUP-RUNTIME && clear"
Enter
Sleep 2s
Show

# Success path: zero todo!/unimplemented! in story-scoped files
Type "grep -c 'todo!\\|unimplemented!' crates/prism-query/src/engine.rs crates/prism-query/src/materialization.rs crates/prism-query/src/internal_tables.rs"
Enter
Sleep 3s

# Error path contrast: boot.rs has deferred steps 9/10 (explicitly out-of-scope)
Type "grep -c 'todo!' crates/prism-bin/src/boot.rs"
Enter
Sleep 2s
Sleep 2s
