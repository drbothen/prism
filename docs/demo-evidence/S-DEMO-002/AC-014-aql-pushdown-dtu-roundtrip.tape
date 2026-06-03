# S-DEMO-002 / AC-014
# AQL filter push-down end-to-end (BC-2.11.007 Mechanism B Verbatim-AQL Passthrough):
#   PrismQL WHERE aql = 'in:devices'
#     -> prism-query seeds FetchContext.query_filters["aql"]
#     -> PipelineExecutor interpolates ${query.filter.aql}
#     -> DTU GET /api/v1/search?aql=in:devices
#     -> DTU /dtu/aql-log confirms "in:devices" received verbatim.
#
# SID-1 unit substitutes (non-ignored):
#   test_BC_2_11_007_armis_aql_pushdown_seeded_in_filter_map
#   test_BC_2_11_007_predicate_tree_to_filter_map_extracts_aql_equality_predicate
#
# E2E: test_BC_2_11_007_e2e_armis_aql_pushdown_devices_dtu_roundtrip
#   Assertion A: non-empty rows (full pipeline succeeds)
#   Assertion B: GET /dtu/aql-log contains "in:devices" verbatim

Output "docs/demo-evidence/S-DEMO-002/AC-014-aql-pushdown-dtu-roundtrip.gif"
Output "docs/demo-evidence/S-DEMO-002/AC-014-aql-pushdown-dtu-roundtrip.webm"

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

Hide
Type "export PATH=/Users/jmagady/.cargo/bin:$PATH"
Enter
Sleep 500ms
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002"
Enter
Sleep 500ms
Show

# AC-014 unit-level: query-layer seeding (SID-1 substitute)
Type "# AC-014 (unit): prism-query AST extracts aql='in:devices' -> FetchContext.query_filters"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-query -E 'test(aql_pushdown)' 2>&1 | grep -E 'PASS|FAIL|tests'"
Enter
Sleep 20s

Type "clear"
Enter
Sleep 500ms

# AC-014 E2E: full pipeline AQL roundtrip including /dtu/aql-log assertion
Type "# AC-014 E2E: WHERE aql='in:devices' -> DTU /dtu/aql-log confirms verbatim receipt"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin --profile e2e --run-ignored all -E 'test(e2e_armis_aql_pushdown)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 25s

Sleep 2s
