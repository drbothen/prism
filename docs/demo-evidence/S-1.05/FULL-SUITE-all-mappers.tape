Output docs/demo-evidence/S-1.05/FULL-SUITE-all-mappers.gif
Output docs/demo-evidence/S-1.05/FULL-SUITE-all-mappers.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# Full S-1.05 test suite — all mapper tests and alias tests pass.
# 68 tests pass (4 pre-existing RED GATE stubs from S-1.04 excluded by design).

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# S-1.05 full suite: CrowdStrike + Cyberint + Claroty + Armis mappers + AliasResolver + VP-017"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf -- tests::mapper_tests tests::alias_tests tests::proptest_extensions 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 30s
