# AC-001: OrgSlug pattern validation — E-CFG-019 rejected for space/unicode/dot/length>64
#
# BC-3.3.004 R-CUST-002 — validate_structural emits E-CFG-019: InvalidOrgSlugPattern
# Story: W3-FIX-CODE-002, branch: feature/W3-FIX-CODE-002, SHA: 85a453d4

Output AC-001-org-slug-pattern-validation.gif
Output AC-001-org-slug-pattern-validation.webm

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
Type "# AC-001: OrgSlug pattern validation (E-CFG-019)"
Enter
Sleep 500ms

Type "# space, unicode, dot, and length>64 all rejected"
Enter
Sleep 500ms

Type "# valid slugs (a-z, 0-9, _, -, up to 64 chars) accepted"
Enter
Sleep 1s

Type "cargo test -p prism-customer-config --test cr003_slug_pattern 2>&1 | tail -20"
Enter
Wait+Screen /test result:/
Sleep 2s
