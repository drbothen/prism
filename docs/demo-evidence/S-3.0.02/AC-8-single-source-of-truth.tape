Output docs/demo-evidence/S-3.0.02/AC-8-single-source-of-truth.gif
Output docs/demo-evidence/S-3.0.02/AC-8-single-source-of-truth.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.0.02 && clear"
Enter
Sleep 2s
Show

Type "# AC-8: DTU_DEFAULT_MODE lives ONLY in prism-core -- zero matches in prism-dtu-* crates"
Enter
Sleep 500ms
Type "grep -RIn 'DTU_DEFAULT_MODE\\|dtu_default_mode' crates/prism-dtu-*/ && echo FOUND || echo 'exit_code=1 (no matches -- PASS)'"
Enter
Sleep 2s
Type "# Now confirm the single centralized definition in prism-core"
Enter
Sleep 500ms
Type "grep -n 'DTU_DEFAULT_MODE' crates/prism-core/src/dtu.rs"
Enter
Sleep 2s
