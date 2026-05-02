# AC-008 (SEC-P2-006): #![deny(deprecated)] compile-time lint gate on prism-sensors
#
# Demonstrates that the crate-level deny attribute is present in lib.rs and that
# the existing #[allow(deprecated)] at the test call site suppresses the promotion.
# Any new caller of init_registry without allow(deprecated) will fail to compile.
#
# BC-3.2.001 invariant 1 — init_registry migration enforced at compile time.
#
# Story: W3-FIX-CODE-004, branch: feature/W3-FIX-CODE-004
# Commits: 360fd53b -> 37088d66 -> cfe585c3

Output AC-008-sec-p2-006-deny-deprecated.gif
Output AC-008-sec-p2-006-deny-deprecated.webm

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

# --- Show deny(deprecated) in lib.rs ---
Type "# AC-008 (SEC-P2-006): crate-level deny(deprecated) in prism-sensors/src/lib.rs"
Enter
Sleep 500ms
Type "grep -n 'deny(deprecated)' crates/prism-sensors/src/lib.rs"
Enter
Sleep 2s

# --- Show allow(deprecated) suppression at test call site ---
Type "# Call site: existing allow(deprecated) suppresses the deny at test scope"
Enter
Sleep 500ms
Type "grep -n 'allow(deprecated)' crates/prism-sensors/tests/test_armis.rs | head -6"
Enter
Sleep 2s
