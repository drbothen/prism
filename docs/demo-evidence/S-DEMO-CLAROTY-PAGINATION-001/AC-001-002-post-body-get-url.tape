# AC-001 / AC-002 — POST body vs GET URL offset/limit dispatch
#
# AC-001: POST steps send offset+limit in request body, URL unchanged (no ?offset=&limit=)
# AC-002: GET steps continue appending ?offset=N&limit=M to URL (regression guard)
#
# Proves:
#   - test_BC_2_16_002_pagination_post_method_url_unchanged  (AC-001 URL side)
#   - test_BC_2_16_002_pagination_post_method_sends_offset_limit_in_body  (AC-001 body side)
#   - test_BC_2_16_002_pagination_get_method_continues_url_params  (AC-002 regression guard)
#
# These are pure unit tests (wiremock) — no external DTU required.
# Traces to: BC-2.16.002 v1.70 §Postconditions "OffsetLimit Pagination Dispatch:
#            POST-body vs GET-URL (DRIFT-D850-001)"

Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/AC-001-002-post-body-get-url.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-PAGINATION-001/AC-001-002-post-body-get-url.webm"

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

Type "# AC-001/AC-002: POST body dispatch + GET URL regression guard"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-spec-engine -E 'test(post_method_url_unchanged) | test(post_method_sends_offset_limit_in_body) | test(get_method_continues_url_params)'"
Enter
Sleep 10s
Sleep 2s
