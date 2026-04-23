Output docs/demo-evidence/S-1.05/AC-007-unmapped-fields-extensions.gif
Output docs/demo-evidence/S-1.05/AC-007-unmapped-fields-extensions.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-7 (BC-2.02.007): Custom vendor field "custom_vendor_field"="xyz" preserved in extensions.
# Covers: all unmapped fields captured, original vendor field names preserved.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-7: Unmapped vendor fields preserved in raw_extensions (never silently dropped)"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_007_custom_vendor_field_preserved_in_extensions test_BC_2_02_007_all_unmapped_fields_captured test_BC_2_02_007_invariant_extensions_use_original_vendor_field_names 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
