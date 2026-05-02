# AC-001: with_failure(slug, DtuType::Claroty, AuthReject) injects ONLY into Claroty
#
# BC-3.6.001 postcondition 2 — other clones (Armis, CrowdStrike, Cyberint) return HTTP 200.
# VP-128 (failure injection isolation), VP-129 (per-DtuType injection scope)
# Story: W3-FIX-CODE-001, branch: feature/W3-FIX-CODE-001, SHA: 8d4a472a

Output AC-001-failure-scope-per-dtu-type.gif
Output AC-001-failure-scope-per-dtu-type.webm

Require cargo

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- Demo begins ---
Type "# AC-001: with_failure honors DtuType -- injection scoped to Claroty only"
Enter
Sleep 500ms

Type "# 4-clone harness (Claroty+Armis+CrowdStrike+Cyberint) for org 'acme'"
Enter
Sleep 500ms

Type "# with_failure('acme', DtuType::Claroty, AuthReject) -> only Claroty gets 401"
Enter
Sleep 500ms

Type "# Armis / CrowdStrike / Cyberint must return 200 (BC-3.6.001 postcondition 2)"
Enter
Sleep 1s

Type "cargo test -p prism-dtu-harness --features dtu --test failure_scope_test test_AC_001 -- --nocapture 2>&1"
Enter
Wait+Screen /test result/
Sleep 2s
