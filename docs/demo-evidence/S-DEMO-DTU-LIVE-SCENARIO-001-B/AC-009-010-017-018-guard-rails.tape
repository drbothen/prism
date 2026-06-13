# AC-009, AC-010, AC-017, AC-018
# Guard Rails — E-DEMO error codes rejected before any clone constructor is called
#
# Covers:
#   AC-009 — E-DEMO-002: mismatched seeds (CS seed=100, Armis seed=200) → Err with both
#              clone names + seeds, before any constructor
#   AC-010 — E-DEMO-003: unrecognized archetype string → Err with clone name + value;
#              also wrong stage_duration_secs length for compromised_endpoint
#   AC-017 — E-DEMO-003: archetype/fixture_set contradiction:
#              (1) "healthy" archetype + scenario.enabled=true (no 5-stage progression)
#              (2) "compromised_endpoint" archetype + fixture_set="dormant" (DormantTenant)
#   AC-018 — E-DEMO-006: mismatched org_ids across scenario-enabled clones → Err with both
#              clone names and both org_id values, before any constructor
# Guard order: E-DEMO-002 → E-DEMO-006 → E-DEMO-003 → E-DEMO-004
#
# Traces to: BC-2.06.019 E-DEMO-002 (PRE-5 / TV-019-012),
#             BC-2.06.019 E-DEMO-003 (PRE-7 / TV-019-013 + EC-019-012),
#             BC-2.06.019 PRE-6 / EC-019-013 / TV-019-015 (E-DEMO-006)
# Story: S-DEMO-DTU-LIVE-SCENARIO-001-B

Output AC-009-010-017-018-guard-rails.gif
Output AC-009-010-017-018-guard-rails.webm

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

Type "# Guard rails: E-DEMO-002/006/003 all fired BEFORE any clone constructor"
Enter
Sleep 500ms
Type "# Guard order: seed-mismatch(002) -> org_id-mismatch(006) -> bad-archetype(003)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-dtu-demo-server --features fixture-gen -E 'test(BC_2_06_019_e_demo_002) | test(BC_2_06_019_e_demo_003) | test(BC_2_06_019_e_demo_006) | test(guard_order)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 6s
Wait+Line /\$/
Sleep 2s
