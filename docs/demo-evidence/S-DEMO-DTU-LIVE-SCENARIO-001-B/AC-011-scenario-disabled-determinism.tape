# AC-011
# INV-SCENARIO-DISABLED-COMPAT-001 — scenario.enabled=false is byte-identical to Story A
#
# Covers:
#   AC-011 — scenario.enabled=false (or absent [clones.*.scenario] block) with seed=42
#              produces responses byte-identical to BC-2.06.018 new_with_seed(42,...) path;
#              timeline: Option<Arc<IncidentTimeline>> is None in clone state.
#              Scenario-enabled path with scenario_start_secs=None: Utc::now() called ONCE,
#              per-run deterministic (not cross-run) — expected behavior, documented in AC-011.
#
# Traces to: BC-2.06.019 INV-SCENARIO-DISABLED-COMPAT-001 (TV-019-007)
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-B

Output AC-011-scenario-disabled-determinism.gif
Output AC-011-scenario-disabled-determinism.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Framerate 30
Set PlaybackSpeed 1

Hide
Type "export PS1='$ '"
Enter
Wait+Line /\$/
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B"
Enter
Wait+Line /\$/
Show

Type "# AC-011: scenario.enabled=false -> byte-identical to BC-2.06.018 seeded path"
Enter
Sleep 500ms
Type "# timeline=None in clone state; Utc::now() called at most once for None start_secs"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-dtu-demo-server --features fixture-gen -E 'test(BC_2_06_019_scenario_disabled_byte_identical_to_seeded_path)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 6s
Wait+Line /\$/
Sleep 2s
