# Demo Evidence — S-1.02: prism-core Entity Types and State Machines

Story: S-1.02 | Branch: feature/S-1.02-entity-types | Commit: 44906b8

## Manifest

### VHS Recordings (AC-1 through AC-8, AC-14, AC-15)

| AC | GIF | WEBM | Tape Script |
|----|-----|------|-------------|
| AC-1 | AC-1-case-status-forward-linear.gif | AC-1-case-status-forward-linear.webm | AC-1-case-status-forward-linear.tape |
| AC-2 | AC-2-case-status-skip-ahead.gif | AC-2-case-status-skip-ahead.webm | AC-2-case-status-skip-ahead.tape |
| AC-3 | AC-3-resolved-to-closed.gif | AC-3-resolved-to-closed.webm | AC-3-resolved-to-closed.tape |
| AC-4 | AC-4-credential-path-traversal.gif | AC-4-credential-path-traversal.webm | AC-4-credential-path-traversal.tape |
| AC-5 | AC-5-credential-null-byte.gif | AC-5-credential-null-byte.webm | AC-5-credential-null-byte.tape |
| AC-6 | AC-6-cursor-cap-exceeded.gif | AC-6-cursor-cap-exceeded.webm | AC-6-cursor-cap-exceeded.tape |
| AC-7 | AC-7-cursor-release-realloc.gif | AC-7-cursor-release-realloc.webm | AC-7-cursor-release-realloc.tape |
| AC-8 | AC-8-alert-severity-ocsf.gif | AC-8-alert-severity-ocsf.webm | AC-8-alert-severity-ocsf.tape |
| AC-14 | AC-14-vp055-storage-atomicity-isolation.gif | AC-14-vp055-storage-atomicity-isolation.webm | AC-14-vp055-storage-atomicity-isolation.tape |
| AC-15 | AC-15-vp057-crash-recovery.gif | AC-15-vp057-crash-recovery.webm | AC-15-vp057-crash-recovery.tape |

### Kani Placeholders (AC-9 through AC-13 — Phase 5 formal verification)

| AC | Placeholder | Proof Harness |
|----|-------------|---------------|
| AC-9 | AC-9-vp005-kani-exactly-12-transitions.md | `proof_exactly_12_transitions` |
| AC-10 | AC-10-vp006-kani-no-self-transitions.md | `proof_no_self_transitions` |
| AC-11 | AC-11-vp011-kani-path-traversal.md | `proof_path_traversal_rejected` |
| AC-12 | AC-12-vp029-kani-cursor-cap.md | `proof_cursor_cap_200` |
| AC-13 | AC-13-vp051-kani-exhaustive-transition-table.md | `proof_exhaustive_transition_table` |

## Full Coverage Report

See [evidence-report.md](evidence-report.md) for AC-to-file mapping, reproduction
commands, and Kani phase gate details.
