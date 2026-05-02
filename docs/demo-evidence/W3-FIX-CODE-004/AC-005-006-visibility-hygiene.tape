# AC-005 (CR-014) + AC-006 (CR-015): visibility hygiene
#
# AC-005: validate_spec_path narrowed from pub to pub(crate) in prism-customer-config.
#         BC-3.3.004 invariant (internal helper not exposed to external crates)
#
# AC-006: Cyberint validate_org_id removed; module doc explains session-routing deviation.
#         BC-3.5.002 precondition 3 (design choice documented, not an omission)
#
# Story: W3-FIX-CODE-004, branch: feature/W3-FIX-CODE-004
# Commits: 360fd53b -> 37088d66 -> cfe585c3

Output AC-005-006-visibility-hygiene.gif
Output AC-005-006-visibility-hygiene.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-004"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- AC-005: validate_spec_path visibility ---
Type "# AC-005 (CR-014): validate_spec_path -- checking current visibility"
Enter
Sleep 500ms
Type "grep -n 'fn validate_spec_path' crates/prism-customer-config/src/validator.rs"
Enter
Sleep 2s

# --- AC-006: Cyberint session-routing model doc ---
Type "# AC-006 (CR-015): Cyberint session routing -- validate_org_id removed, doc explains why"
Enter
Sleep 500ms
Type "grep -n 'validate_org_id\|incompatible' crates/prism-dtu-cyberint/src/routes/alerts.rs | head -6"
Enter
Sleep 2s
