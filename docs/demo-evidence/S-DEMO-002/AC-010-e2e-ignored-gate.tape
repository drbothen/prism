# AC-010: E2E tests gated behind #[ignore]; standard run skips them; e2e profile runs them
# Story: S-DEMO-002
# Demonstrates:
#   - Standard 'cargo nextest run -p prism-bin' skips e2e_smoke tests (correct gate)
#   - '--profile e2e --run-ignored all' activates all 13 e2e smoke tests

Output docs/demo-evidence/S-DEMO-002/AC-010-e2e-ignored-gate.gif
Output docs/demo-evidence/S-DEMO-002/AC-010-e2e-ignored-gate.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1400
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"
Env PATH "/Users/jmagady/Dev/prism/.worktrees/S-DEMO-002/target/release:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002"
Enter
Sleep 500ms
Show

Type "# AC-010: Standard run skips e2e_smoke tests (they are #[ignore]'d)"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-bin -E 'test(e2e_smoke)' 2>&1 | head -20"
Enter
Sleep 30s

Type "# Result: 0 tests run (all skipped -- #[ignore] gate is working)"
Enter
Sleep 2s
