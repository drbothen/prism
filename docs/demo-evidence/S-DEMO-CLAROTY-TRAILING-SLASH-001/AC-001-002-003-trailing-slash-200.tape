# AC-001/AC-002/AC-003 — Trailing-slash POST paths return HTTP 200
#
# Proves POST /api/v1/alerts/, /api/v1/devices/, /api/v1/audit_log/get/ all resolve
# to their handlers (HTTP 200 + fixture body) after NormalizePathLayer::trim_trailing_slash()
# strips the inbound trailing slash before axum route resolution.
#
# Demo vehicle: cargo nextest. The harness starts a live ClarotyClone on an ephemeral port,
# fires real reqwest HTTP POST requests with Bearer auth and JSON body, and asserts:
#   - HTTP 200 status code
#   - Response body contains expected fixture key ("alerts", "devices", or audit entries)
#
# Traces to: BC-2.16.013 v1.25 §Postconditions §1

Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-001-002-003-trailing-slash-200.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-001-002-003-trailing-slash-200.webm"

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-TRAILING-SLASH-001"
Enter
Sleep 1s
Show

Type "# AC-001/002/003: POST trailing-slash routes → HTTP 200 (live DTU clone)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-claroty --features dtu -E 'test(claroty_trailing_slash)'"
Enter
Sleep 30s

Sleep 2s
