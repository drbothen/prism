Output docs/demo-evidence/S-3.1.01/AC-002-display-format.gif
Output docs/demo-evidence/S-3.1.01/AC-002-display-format.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.01 && clear"
Enter
Sleep 2s
Show

Type "# AC-4: OrgId Display — hyphenated lowercase UUID (BC-3.1.001 invariant 3)"
Enter
Sleep 500ms
Type "cargo test --test bc_3_1_001_org_id test_bc_3_1_001_ac_4_display_hyphenated_lowercase -- --nocapture 2>&1"
Enter
Sleep 15s
