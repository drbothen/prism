# AC-003 / AC-005 — Multi-page offset advances + first-page offset=0
#
# AC-003: Multi-page Claroty query returns >100 rows — wiremock companion (SID-1)
#   test_BC_2_16_002_pagination_post_offset_advances_across_pages
#   (2-page wiremock; page-2 body verified to contain "offset": 51)
#   NOTE: The live-DTU integration test
#   test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data
#   is #[ignore]'d per DTU-EXT-001 (requires prism-dtu-claroty clone + 102-entry fixture).
#   The wiremock companion provides equivalent non-ignored coverage of the pagination
#   loop per SID-1 (cite: CLAUDE.md §Standing Adversary Probes SID-1).
#
# AC-005: First page uses offset=0 in body
#   test_BC_2_16_002_pagination_post_first_page_offset_zero_in_body
#
# Both tests use wiremock; no external DTU required.
# Traces to: BC-2.01.013 v1.14 postcondition §1 (AC-003), BC-2.16.002 v1.70 §Postconditions
#            offset initialization clause (AC-005)

Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/AC-003-005-offset-advances-first-page-zero.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/AC-003-005-offset-advances-first-page-zero.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-PAGINATION-001"
Enter
Sleep 1s
Show

Type "# AC-003/AC-005: offset advances across pages + first page offset=0 in body"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-spec-engine -E 'test(post_offset_advances_across_pages) | test(post_first_page_offset_zero_in_body)'"
Enter
Sleep 10s
Sleep 2s
