# AC-QERR-001 — query tool sensor_errors wire surfaces per-target HTTP error detail
# AC-QERR-002 — cross-client partial-failure regression guard (expected GREEN)
#
# Proves that when the query tool encounters a sensor HTTP failure, the serialized MCP
# response carries a non-empty sensor_errors array with entries in the form:
#   "<table>: HTTP <status>: <sanitized-body-snippet>"   (non-empty body)
#   "<table>: HTTP <status>"                              (empty body, status-only)
# And that sensor_errors is ABSENT on success (not null, not []).
#
# Two tests are shown:
#   test_BC_2_11_001_query_sensor_errors_surfaces_per_target_http_detail  (RG-016, AC-QERR-001)
#     EC-11-088: 403 + body → "claroty_devices: HTTP 403: <body>"
#     EC-11-089: 503 empty body → "claroty_devices: HTTP 503"
#     Absence invariant: success response has no sensor_errors key
#   test_BC_2_11_001_EC_11_091_cross_client_partial_failure_sensor_errors_http_format (RG-017, AC-QERR-002)
#     clients=["org-a","org-b"]: org-a 403 → sensor_errors; org-b success → rows
#
# Demo vehicle: cargo nextest run against prism-mcp integration tests.
# Traces to: BC-2.11.001 §Postconditions sensor_errors wire; EC-11-088/089/090/091

Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-QERR-001-query-sensor-errors.gif"
Output "docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/AC-QERR-001-query-sensor-errors.webm"

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
Enter
Sleep 1s
Show

Type "# AC-QERR-001/002: query sensor_errors wire — per-target HTTP error detail + cross-client"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_11_001_query_sensor_errors) | test(BC_2_11_001_EC_11_091)'"
Enter
Sleep 30s

Sleep 2s
