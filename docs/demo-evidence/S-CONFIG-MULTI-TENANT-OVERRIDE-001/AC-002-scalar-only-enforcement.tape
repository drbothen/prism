Output AC-002-scalar-only-enforcement.gif
Output AC-002-scalar-only-enforcement.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CONFIG-MULTI-TENANT-OVERRIDE-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-002: Scalar-only overlay enforcement (BC-2.06.013)"
Enter
Sleep 500ms

Type "# ERROR PATHS: [[tables]] -> E-SPEC-021 | auth_type -> E-SPEC-023 | wrong instance_id -> E-SPEC-020"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_06_013)' 2>&1 | tail -10"
Enter
Sleep 10s
