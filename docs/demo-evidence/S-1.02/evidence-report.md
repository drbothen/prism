# Evidence Report — S-1.02: prism-core Entity Types and State Machines

**Branch:** feature/S-1.02-entity-types
**Commit at recording:** 44906b8
**Test suite:** 103/103 passing (104 with proptest iterations counted as 1)
**Recording tool:** VHS v0.10.0
**Date:** 2026-04-22

---

## Coverage Summary

| AC | Title | Format | Evidence | Status |
|----|-------|--------|----------|--------|
| AC-1 | CaseStatus: New → Acknowledged (forward linear) | VHS GIF+WEBM | `AC-1-case-status-forward-linear.gif` | RECORDED |
| AC-2 | CaseStatus: New → Closed (skip-ahead) | VHS GIF+WEBM | `AC-2-case-status-skip-ahead.gif` | RECORDED |
| AC-3 | CaseStatus: Resolved → Closed (state machine returns true) | VHS GIF+WEBM | `AC-3-resolved-to-closed.gif` | RECORDED |
| AC-4 | CredentialName: "../../passwd" rejected | VHS GIF+WEBM | `AC-4-credential-path-traversal.gif` | RECORDED |
| AC-5 | CredentialName: null byte rejected | VHS GIF+WEBM | `AC-5-credential-null-byte.gif` | RECORDED |
| AC-6 | CursorRegistry: 201st allocation returns Err | VHS GIF+WEBM | `AC-6-cursor-cap-exceeded.gif` | RECORDED |
| AC-7 | CursorRegistry: release + re-allocate returns Ok | VHS GIF+WEBM | `AC-7-cursor-release-realloc.gif` | RECORDED |
| AC-8 | AlertSeverity::Critical → OCSF severity_id 5 | VHS GIF+WEBM | `AC-8-alert-severity-ocsf.gif` | RECORDED |
| AC-9 | VP-005 Kani: exactly 12 transitions | Markdown placeholder | `AC-9-vp005-kani-exactly-12-transitions.md` | PHASE-5-GATE |
| AC-10 | VP-006 Kani: no self-transitions | Markdown placeholder | `AC-10-vp006-kani-no-self-transitions.md` | PHASE-5-GATE |
| AC-11 | VP-011 Kani: path traversal rejected | Markdown placeholder | `AC-11-vp011-kani-path-traversal.md` | PHASE-5-GATE |
| AC-12 | VP-029 Kani: cursor cap 200 | Markdown placeholder | `AC-12-vp029-kani-cursor-cap.md` | PHASE-5-GATE |
| AC-13 | VP-051 Kani: exhaustive 5×5 transition table | Markdown placeholder | `AC-13-vp051-kani-exhaustive-transition-table.md` | PHASE-5-GATE |
| AC-14 | VP-055 proptest: batch atomicity + domain isolation | VHS GIF+WEBM | `AC-14-vp055-storage-atomicity-isolation.gif` | RECORDED |
| AC-15 | VP-057 unit test: crash recovery denylist at 3 | VHS GIF+WEBM | `AC-15-vp057-crash-recovery.gif` | RECORDED |

**Recorded:** 10/15 ACs
**Kani placeholders:** 5/15 ACs (AC-9 through AC-13)

---

## VHS Recordings — File Manifest

Each recorded AC has three artifacts: `.tape` (source script), `.gif` (embed), `.webm` (archival).

| AC | Tape | GIF | WEBM | Test Name |
|----|------|-----|------|-----------|
| AC-1 | `AC-1-case-status-forward-linear.tape` | `AC-1-case-status-forward-linear.gif` | `AC-1-case-status-forward-linear.webm` | `test_BC_S_02_001_ac1_new_to_acknowledged_is_valid` |
| AC-2 | `AC-2-case-status-skip-ahead.tape` | `AC-2-case-status-skip-ahead.gif` | `AC-2-case-status-skip-ahead.webm` | `test_BC_S_02_001_ac2_new_to_closed_is_valid` |
| AC-3 | `AC-3-resolved-to-closed.tape` | `AC-3-resolved-to-closed.gif` | `AC-3-resolved-to-closed.webm` | `test_BC_S_02_001_ac3_resolved_to_closed_state_machine_returns_true` |
| AC-4 | `AC-4-credential-path-traversal.tape` | `AC-4-credential-path-traversal.gif` | `AC-4-credential-path-traversal.webm` | `test_BC_S_02_003_ac4_rejects_path_traversal_double_dot` |
| AC-5 | `AC-5-credential-null-byte.tape` | `AC-5-credential-null-byte.gif` | `AC-5-credential-null-byte.webm` | `test_BC_S_02_003_ac5_rejects_null_byte` |
| AC-6 | `AC-6-cursor-cap-exceeded.tape` | `AC-6-cursor-cap-exceeded.gif` | `AC-6-cursor-cap-exceeded.webm` | `test_BC_S_02_004_ac6_201st_allocation_fails` |
| AC-7 | `AC-7-cursor-release-realloc.tape` | `AC-7-cursor-release-realloc.gif` | `AC-7-cursor-release-realloc.webm` | `test_BC_S_02_004_ac7_release_then_allocate_succeeds` |
| AC-8 | `AC-8-alert-severity-ocsf.tape` | `AC-8-alert-severity-ocsf.gif` | `AC-8-alert-severity-ocsf.webm` | `test_BC_S_02_002_ac8_critical_ocsf_severity_id_is_5` |
| AC-14 | `AC-14-vp055-storage-atomicity-isolation.tape` | `AC-14-vp055-storage-atomicity-isolation.gif` | `AC-14-vp055-storage-atomicity-isolation.webm` | `cargo test -p prism-storage` (proptest VP-055) |
| AC-15 | `AC-15-vp057-crash-recovery.tape` | `AC-15-vp057-crash-recovery.gif` | `AC-15-vp057-crash-recovery.webm` | `test_BC_S_02_vp057` (crash recovery unit tests) |

---

## Kani Placeholder Details

ACs 9-13 require `cargo kani` and are gated to Phase 5 (Formal Verification). Each
placeholder document contains:

- The exact `cargo kani --harness <name>` reproduction command
- The proof file location and proof function name
- The runtime unit tests that exercise the same property (all passing)

| AC | Proof Harness | Proof File | Runtime Coverage |
|----|--------------|------------|------------------|
| AC-9 | `proof_exactly_12_transitions` | `crates/prism-core/src/proofs/case_status.rs` | `test_BC_S_02_001_vp005_exactly_12_valid_transitions` |
| AC-10 | `proof_no_self_transitions` | `crates/prism-core/src/proofs/case_status.rs` | `test_BC_S_02_001_vp006_no_self_transitions` |
| AC-11 | `proof_path_traversal_rejected` | `crates/prism-core/src/proofs/credential_name.rs` | 5 tests in `test_credential_name.rs` |
| AC-12 | `proof_cursor_cap_200` | `crates/prism-core/src/proofs/cursor.rs` | 4 tests in `test_cursor_registry.rs` |
| AC-13 | `proof_exhaustive_transition_table` | `crates/prism-core/src/proofs/case_status_exhaustive.rs` | `test_BC_S_02_001_vp051_exhaustive_25_pairs_correct_outcome` |

---

## Reproduction

To reproduce all VHS recordings from scratch:

```sh
cd /path/to/prism/.worktrees/S-1.02-entity-types

# Pre-build to warm the cargo cache (optional but makes recordings faster)
cargo build -p prism-core -p prism-storage

# Run VHS for each AC (from worktree root)
for tape in docs/demo-evidence/S-1.02/AC-{1,2,3,4,5,6,7,8,14,15}-*.tape; do
  vhs "$tape"
done
```

To run the Kani proofs (Phase 5):

```sh
cd crates/prism-core
cargo kani --harness proof_exactly_12_transitions
cargo kani --harness proof_no_self_transitions
cargo kani --harness proof_path_traversal_rejected
cargo kani --harness proof_cursor_cap_200
cargo kani --harness proof_exhaustive_transition_table
```
