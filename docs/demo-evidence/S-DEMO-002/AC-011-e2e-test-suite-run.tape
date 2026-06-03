# Full e2e test suite run: 108 pass, 13 e2e_smoke fail (env blocker documented)
# Story: S-DEMO-002
# Demonstrates:
#   - All 108 non-e2e tests pass (Green baseline)
#   - 13 e2e_smoke tests fail with consistent "Broken pipe" error (env vars not set)
#   - This is the environmental blocker documented in the evidence report

Output docs/demo-evidence/S-DEMO-002/AC-011-e2e-test-suite-run.gif
Output docs/demo-evidence/S-DEMO-002/AC-011-e2e-test-suite-run.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 12
Set Width 1400
Set Height 800
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"
Env PATH "/Users/jmagady/Dev/prism/.worktrees/S-DEMO-002/target/release:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002"
Enter
Sleep 500ms
Show

Type "# E2E suite: 108 pass, 13 e2e_smoke fail (env/plugin blocker)"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-bin --profile e2e --run-ignored all 2>&1 | tail -30"
Enter
Sleep 90s

Type "# Summary: 108 pass, 13 e2e_smoke fail (env vars + plugin manifest gap in harness)"
Enter
Sleep 2s
