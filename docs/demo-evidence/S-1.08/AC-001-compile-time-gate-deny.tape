Output docs/demo-evidence/S-1.08/AC-001-compile-time-gate-deny.gif
Output docs/demo-evidence/S-1.08/AC-001-compile-time-gate-deny.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-1 (BC-2.04.001, BC-2.04.004)
# Compile-time gate: binary built WITHOUT crowdstrike-write → DeniedCompileTime
# All write gate functions return CompileTimeGate::Absent (real cfg gate).

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1: Compile-time gate absent — all four write gates return Deny"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_BC_2_04_001 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
