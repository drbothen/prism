# AC-003: Tier-2 column query raises E-QUERY-038; available_columns contains
#         raw_extensions not raw Tier-2 name
#
# AC-003: PrismQL SELECT device_uid raises E-QUERY-038 at plan time.
#         available_columns MUST contain raw_extensions, finding_info_title, time, class_uid, _sensor.
#         MUST NOT contain device_uid as standalone column.
#         Authoritative test: test_BC_2_16_017_..._e2e_e_query_038_tier2_device_uid (prism-bin)
#         via QueryEngine::execute (end-to-end plan-time gate).
#
# Tests: RG-003 authoritative (test_BC_2_16_017_...e2e_e_query_038_tier2_device_uid) — prism-bin
#        RG-003 defense-in-depth (test_BC_2_16_017_...tier2_column_excluded_from_projection) — prism-sensors
# Traces to: BC-2.16.017 §Invariants, EC-016-017-006

Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-003-plan-gate-tests.gif"
Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-003-plan-gate-tests.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 180s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-DEVVULNREL-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-003: E-QUERY-038 plan gate — SELECT device_uid (Tier-2 column) (RG-003)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin -E 'test(e2e_e_query_038)' --no-fail-fast 2>&1 | tail -15`
Enter
Sleep 120s
Sleep 2s
