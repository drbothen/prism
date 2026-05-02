# AC-006: org_slug cross-check — Matched/Mismatched/OrgNotInRegistry paths
#
# BC-3.1.002 postcondition (SEC-007)
# validate_org_slug_cross_check: returns SlugCheckResult enum variant
# OrgNotInRegistry: tracing::warn (no panic) — audit-must-not-fail semantics
# Story: W3-FIX-CODE-002, branch: feature/W3-FIX-CODE-002, SHA: 85a453d4

Output AC-006-org-slug-cross-check.gif
Output AC-006-org-slug-cross-check.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 120s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-002"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- Demo begins ---
Type "# AC-006: org_slug cross-check against OrgRegistry (SEC-007)"
Enter
Sleep 500ms

Type "# Matched: slug == registry entry -> no warning"
Enter
Sleep 500ms

Type "# Mismatched: wrong slug -> SlugCheckResult::Mismatched (not panic)"
Enter
Sleep 500ms

Type "# OrgNotInRegistry: unknown org_id -> SlugCheckResult::OrgNotInRegistry (not panic)"
Enter
Sleep 1s

Type "cargo test -p prism-audit --test sec007_org_slug_cross_check 2>&1 | tail -20"
Enter
Wait+Screen /test result:/
Sleep 2s
