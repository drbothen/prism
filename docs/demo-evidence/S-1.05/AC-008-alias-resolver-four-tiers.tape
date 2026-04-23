Output docs/demo-evidence/S-1.05/AC-008-alias-resolver-four-tiers.gif
Output docs/demo-evidence/S-1.05/AC-008-alias-resolver-four-tiers.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-8 (BC-2.02.008): AliasResolver four-tier resolution.
# Tier 1: source_sensor → PrismMetadata
# Tier 2: device.hostname → ProtoField (recursive descent)
# Tier 3: custom_vendor_field → RawExtension
# Tier 4: absent field → AliasResult::Absent

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-8: AliasResolver four-tier resolution (Prism metadata → proto → raw_extensions → Absent)"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf -- tests::alias_tests 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
