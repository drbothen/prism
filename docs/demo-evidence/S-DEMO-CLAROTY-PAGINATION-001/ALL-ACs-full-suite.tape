# ALL ACs — Full pagination test suite (9 tests: 8 pass + 1 #[ignore]'d DTU integration)
#
# Runs all BC_2_16_002_pagination tests:
#   AC-001: test_BC_2_16_002_pagination_post_method_url_unchanged
#   AC-001: test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body
#   AC-002: test_BC_2_16_002_pagination_get_method_continues_url_params
#   AC-003 (wiremock): test_BC_2_16_002_pagination_post_offset_advances_across_pages
#   AC-004: test_BC_2_16_002_pagination_body_template_merge_preserves_existing_keys
#   AC-005: test_BC_2_16_002_pagination_post_first_page_offset_zero_in_body
#   EC-002: test_BC_2_16_002_pagination_post_non_object_body_surfaces_error
#   EC-002: test_BC_2_16_002_pagination_post_invalid_json_body_surfaces_error
#   AC-006: test_BC_2_16_002_pagination_build_paged_url_for_test_get_path_still_works
#   [SKIP] test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data
#          #[ignore = "DTU-EXT-001: requires prism-dtu-claroty clone + 102-entry fixture"]
#
# Summary shows: 9 tests run: 9 passed, N skipped (the ignored test is skipped by default).
# For AC-003 live-DTU coverage see evidence-report.md §DTU-EXT-001 gating note.
#
# Traces to: BC-2.16.002 v1.70, BC-2.16.013 v1.25, BC-2.01.013 v1.14

Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/ALL-ACs-full-suite.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/ALL-ACs-full-suite.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-PAGINATION-001"
Enter
Sleep 1s
Show

Type "# S-DEMO-CLAROTY-PAGINATION-001: full POST-body OffsetLimit pagination suite"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_002_pagination)'"
Enter
Sleep 10s
Sleep 2s
