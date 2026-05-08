---
document_type: code-review-report
review: PR #135 code review (cognitive diversity)
target_pr: 135
target_sha: 65411ea4
diff_base: 7c413692
reviewer: code-reviewer
review_date: 2026-05-08
---

# PR #135 Code Review

## Verdict
**APPROVED WITH NITS** — 5 findings (3 MED + 2 LOW), none blocking.

## Findings

### CR-001 (LOW, code-quality): Redundant iteration in write_pipeline.rs:350-355
- `would_affect_count` and `total_rows` computed via 2 iterations of identical values. Compute once and reuse.

### CR-002 (MED, code-quality): Dead WriteUnbounded guard in safety_check.rs:264-269
- Tail of `check_structural_batch_limit` has structurally unreachable WriteUnbounded guard given gate ordering in phase2_safety_check. Existing comment admits this; dead branch is maintenance hazard producing semantically wrong error type. Replace with `debug_assert!` precondition.

### CR-003 (MED, maintainability): Internal Rust path leak in adapter.rs:363
- Default `write()` trait body uses `std::any::type_name::<Self>()` for `sensor` field of `WriteNotImplemented` — leaks fully-qualified internal Rust type paths into MCP-boundary error messages. Use `self.sensor_name()` which exists for this purpose.

### CR-004 (LOW, code-quality): Duplicate field in WritePreview
- `write_result.rs` `WritePreview` has both `risk_tier` and `reversibility` fields always set to same value. Pure duplication; remove `reversibility` or derive from `risk_tier` at serialization.

### CR-005 (MED, pattern-consistency): Overbroad clippy suppression in write_table_registration.rs:33
- Module-level `#![allow(dead_code, unused_variables)]` is overbroad. Individual `_`-prefixed parameters in stub methods already silence the warnings. Removing module-level suppression restores clippy's ability to catch genuine future issues.

## KUDOs
- Phase 5 ordering documentation rationale block
- SensorError::error_code() exhaustive-match defense in depth
- E-QUERY-026/029/030 architectural distinction prose
- Bounded fan-out error allocation (Phase 4 batch limit)

## Convergence Notes
- No CRITICAL or HIGH findings
- 5 nits should be addressed before merge for cleanest code
