# S-DEMO-002 / AC-001 + AC-002 + AC-010
# AC-001: DTU demo server + prism-bin launch without error (BC-2.22.001).
# AC-002: tools/list returns `query` tool present (BC-2.10.001).
# AC-010: #[ignore] gate — standard profile skips 13 E2E tests; e2e profile runs them.
#
# Red Gate test: test_BC_2_22_001_e2e_smoke_test_launches_dtu_and_prism_bin_without_error

Output "docs/demo-evidence/S-DEMO-002/AC-001-010-e2e-launch-ignore-gate.gif"
Output "docs/demo-evidence/S-DEMO-002/AC-001-010-e2e-launch-ignore-gate.webm"

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Sleep 2s

# Setup: PATH and workdir
Hide
Type "export PATH=/Users/jmagady/.cargo/bin:$PATH"
Enter
Sleep 500ms
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002"
Enter
Sleep 500ms
Show

# AC-010: standard nextest run SKIPS E2E tests (13 skipped, no e2e_smoke tests run)
Type "# AC-010: standard nextest run — E2E tests are #[ignore]d (13 skipped)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin 2>&1 | tail -2"
Enter
Sleep 50s

Type "clear"
Enter
Sleep 500ms

# AC-001 + AC-002: E2E profile un-ignores the test — both subprocesses launch, MCP handshake succeeds
Type "# AC-001/002: e2e profile — subprocess launch + MCP tools/list has 'query'"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin --profile e2e --run-ignored all -E 'test(e2e_smoke_test_launches)' 2>&1 | tail -4"
Enter
Sleep 30s

Sleep 2s
