# AC-004 (CR-020): validate_spec_path pub-vs-pub(crate) deviation comment
#
# Confirms that the deviation comment block is present immediately above
# #[doc(hidden)] on validate_spec_path in prism-customer-config/src/validator.rs.
# The comment explains why pub + #[doc(hidden)] is the correct idiom over pub(crate).
#
# BC-3.3.004 invariant 1 (deviation documented at call site)
# Story: W3-FIX-CODE-005, branch: feature/W3-FIX-CODE-005, SHA: 652409cf

Output AC-004-cr020-validate-spec-path-deviation-comment.gif
Output AC-004-cr020-validate-spec-path-deviation-comment.webm

Require grep

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 60s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-005"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- AC-004: show the deviation comment above #[doc(hidden)] ---
Type "# AC-004 (CR-020): deviation comment above #[doc(hidden)] on validate_spec_path"
Enter
Sleep 500ms
Type "grep -n -A10 'AC-005 deviation' crates/prism-customer-config/src/validator.rs | head -14"
Enter
Sleep 2s
