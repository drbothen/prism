Output docs/demo-evidence/S-1.05/AC-010-vp017-proptest-no-silent-drop.gif
Output docs/demo-evidence/S-1.05/AC-010-vp017-proptest-no-silent-drop.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-10 (VP-017): No input fields silently dropped across 10,000+ generated test cases.
# Property: for every Ok normalization result, all input keys appear in mapped fields
#           or raw_extensions. BC-2.02.007 invariant enforced by proptest.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-10: VP-017 proptest — no fields silently dropped across generated sensor records"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf -- tests::proptest_extensions 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
