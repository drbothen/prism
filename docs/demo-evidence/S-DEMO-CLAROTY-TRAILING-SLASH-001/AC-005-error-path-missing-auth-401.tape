# AC-005 / EC-002 — Error path: POST /api/v1/alerts/ without auth → 401 (not 404)
#
# Critical placement proof for NormalizePathLayer:
#   404 = layer is mis-placed (Router::layer no-ops in axum 0.7 — routing before inner layers)
#   401 = layer is at outer-service level — strip ran BEFORE routing, route matched, auth fired
#
# 401 is the proof that normalize_path runs before routing resolves.
#
# Traces to: BC-2.16.013 v1.25 §Postconditions §1; EC-002.

Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-005-error-path-missing-auth-401.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/AC-005-error-path-missing-auth-401.webm"

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

Type "# EC-002: POST /api/v1/alerts/ with NO auth → 401 (layer ran BEFORE routing → 401 not 404)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-dtu-claroty --features dtu -E 'test(trailing_slash_alerts_missing_auth)'"
Enter
Sleep 30s

Sleep 2s
