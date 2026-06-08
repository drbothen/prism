# AC-005 — Backward compatibility: no-slash routes + tags route + /dtu/health/ unbroken
#
# Proves that NormalizePathLayer::trim_trailing_slash() is STRIP-ONLY:
#   EC-001: POST /api/v1/alerts (no slash) → HTTP 200
#   EC-001: POST /api/v1/devices (no slash) → HTTP 200
#   AC-005 intentional tags route: POST /api/v1/devices/:id/tags/ → HTTP 201
#   EC-003: GET /dtu/health/ (trailing slash) → HTTP 200
#
# Traces to: BC-2.16.013 v1.25 §Postconditions §1 — backward-compat clause.

Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-005-backward-compat-no-slash-tags-health.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-005-backward-compat-no-slash-tags-health.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-TRAILING-SLASH-001"
Enter
Sleep 1s
Show

Type "# AC-005: backward compat — no-slash routes, tags route, /dtu/health/ all unbroken"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-dtu-claroty --features dtu -E 'test(BC_2_16_013_no_slash) | test(BC_2_16_013_tags) | test(BC_2_16_013_dtu_health)'"
Enter
Sleep 30s

Sleep 2s
