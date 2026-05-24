Output AC-004-org-registry-cross-validation.gif
Output AC-004-org-registry-cross-validation.webm

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

Type "# AC-004: OrgRegistry cross-validation at boot (BC-2.06.015)"
Enter
Sleep 500ms

Type "# ERROR PATH: customers/unknown-org/ dir not in OrgRegistry -> E-SPEC-022 boot abort"
Enter
Sleep 300ms
Type "# Asserts: result.errors has E-SPEC-022 | msg contains slug | resolved map empty"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_06_015_unknown_org_dir_aborts_boot_with_e_spec_022)' 2>&1 | tail -8"
Enter
Sleep 10s
