# AC-003 — Unset CROWDSTRIKE_BASE_URL → E-SPEC-024 structured error, not panic
#
# Story:    S-DEMO-CROWDSTRIKE-MULTIREGION-001 v1.4
# AC:       AC-003 (BC-2.16.009 §VR6 + E-SPEC-024; EC-009-009)
# Evidence: Red Gate test passes: unset env var → Err(SpecEngineError), not panic.

Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-003-unset-env-e-spec-024.gif
Output docs/demo-evidence/S-DEMO-CROWDSTRIKE-MULTIREGION-001/AC-003-unset-env-e-spec-024.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CROWDSTRIKE-MULTIREGION-001 && clear"
Enter
Sleep 1s
Show

Type "cargo nextest run -p prism-spec-engine -E test(BC_2_16_013_crowdstrike_base_url_env_unset_returns_spec_error_not_panic) 2>&1"
Enter
Sleep 600s
