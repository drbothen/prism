Output docs/demo-evidence/S-1.05/AC-005-claroty-integer-id-uid.gif
Output docs/demo-evidence/S-1.05/AC-005-claroty-integer-id-uid.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-5 (BC-2.02.005): Claroty asset id=42 (integer) → device.uid="claroty:42".
# Also covers: string ID polymorphism and unknown record type error path.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-5: Claroty integer id=42 → device.uid='claroty:42' (polymorphic ID)"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_005_claroty_integer_id_converts_to_uid_string test_BC_2_02_005_claroty_string_id_converts_to_uid_string test_BC_2_02_005_claroty_unknown_record_type_returns_error 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
