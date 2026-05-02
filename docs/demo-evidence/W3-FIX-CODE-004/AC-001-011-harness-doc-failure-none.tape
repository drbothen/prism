# AC-001 (CR-010) + AC-002 (CR-011): harness module doc accuracy + with_failure(None) clears entry
#
# AC-001: Module doc at harness.rs:18-20 accurately describes post-CR-002 Drop behavior
#         (no more "calls handle.abort()"; now describes graceful-shutdown drain)
#         BC-3.5.001 EC-004
#
# AC-002: with_failure(slug, dtu_type, FailureMode::None) calls HashMap::remove, not insert.
#         A second call with None leaves initial_failure.is_empty() == true.
#         BC-3.6.001 invariant 4
#
# Story: W3-FIX-CODE-004, branch: feature/W3-FIX-CODE-004
# Commits: 360fd53b -> 37088d66 -> cfe585c3

Output AC-001-011-harness-doc-failure-none.gif
Output AC-001-011-harness-doc-failure-none.webm

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

# --- AC-001: show the updated module doc ---
Type "# AC-001 (CR-010): harness.rs module doc -- no longer references handle.abort()"
Enter
Sleep 500ms
Type "grep -A4 'Drop behavior' crates/prism-dtu-harness/src/harness.rs"
Enter
Sleep 2s

Type "# AC-002 (CR-011): with_failure(None) removes entry -- 4 invariant-4 tests pass"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-harness --features dtu --test cr011_failure_mode_none_clears 2>&1 | tail -6"
Enter
Sleep 30s
