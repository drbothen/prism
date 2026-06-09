# AC-004 / EC-002 — Body merge preserves existing keys + error paths
#
# AC-004: Body template merge preserves existing body fields
#   test_BC_2_16_002_pagination_body_template_merge_preserves_existing_keys
#   Verifies offset+limit injection does NOT clobber pre-existing body_template keys.
#
# EC-002 (success path + error path):
#   test_BC_2_16_002_pagination_post_non_object_body_surfaces_error
#     → POST body_template is a non-object JSON value → surfaces SpecEngineError, no panic
#   test_BC_2_16_002_pagination_post_invalid_json_body_surfaces_error
#     → POST body_template is invalid JSON → surfaces SpecEngineError, no panic
#
# AC-006 (regression guard for build_paged_url_for_test GET path):
#   test_BC_2_16_002_pagination_build_paged_url_for_test_get_path_still_works
#     → test helper still returns URL-appended results for GET steps (no signature change)
#
# All tests are pure unit tests (wiremock); no external DTU required.
# Traces to: BC-2.16.002 v1.70 §Postconditions body merge clause (AC-004),
#            EC-002 (non-object/invalid body → Err, no panic), AC-006

Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/AC-004-EC002-body-merge-error-paths.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/AC-004-EC002-body-merge-error-paths.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 520
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-PAGINATION-001"
Enter
Sleep 1s
Show

Type "# AC-004/EC-002/AC-006: body merge + error paths + GET regression guard"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-spec-engine -E 'test(body_template_merge_preserves_existing_keys) | test(post_non_object_body_surfaces_error) | test(post_invalid_json_body_surfaces_error) | test(build_paged_url_for_test_get_path_still_works)'"
Enter
Sleep 10s
Sleep 2s
