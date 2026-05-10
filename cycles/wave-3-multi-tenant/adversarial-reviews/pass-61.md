---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T15:30:00
phase: 3
inputs:
  - "crates/prism-query/src/engine.rs"
  - "crates/prism-query/src/materialization.rs"
  - "crates/prism-query/src/internal_tables.rs"
  - "crates/prism-query/src/explain.rs"
  - "crates/prism-query/src/pushdown.rs"
  - "crates/prism-query/tests/execute_integration_tests.rs"
  - "crates/prism-core/src/error.rs"
  - "crates/prism-sensors/src/registry.rs"
  - ".factory/stories/S-3.02-FOLLOWUP-RUNTIME-query-engine.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.001"
  - ".factory/specs/behavioral-contracts/BC-2.11.006"
  - ".factory/specs/behavioral-contracts/BC-2.15.011"
input-hash: "a3d9f1e"
traces_to: prd.md
pass: 61
previous_review: "pass-60.md"
review_class: PR-LEVEL
scope: PR #141 — S-3.02-FOLLOWUP-RUNTIME — feature/S-3.02-FOLLOWUP-RUNTIME vs origin/develop (Pass 4, idempotency check, post pass-60 CLEAN)
---

# Adversarial Review: PR #141 S-3.02-FOLLOWUP-RUNTIME — QueryEngine Execution Pipeline (Pass 61)

## Finding ID Convention

Finding IDs use the format: `ADV-W3MT-P61-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `W3MT`: wave-3-multi-tenant cycle
- `P61`: Pass 61
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

## Scope Note

This is PR-LEVEL Pass 4 — a fresh-context idempotency check targeting convergence streak 2/3.
Pass 60 (PR-level Pass 3) was CLEAN with 4 non-blocking observations (2 MED, 2 LOW).
The MED/LOW findings from pass-60 are NOT closed — they remain open as tracked non-blockers.
This pass is conducted with information asymmetry: the adversary reviewed the current diff
and source independently without consulting prior pass findings before writing this review.

**Closed findings from prior PR passes (DO NOT RE-RAISE):**
- Pass 58 (PR-P01): CRIT-001, CRIT-002, HIGH-001..005, MED-001..004, LOW-001..002 — ALL CLOSED
- Pass 59 (PR-P02): CRIT-001, HIGH-001..002, MED-001, LOW-001..002 — ALL CLOSED
- Pass 60 (PR-P03): 0 CRIT, 0 HIGH (CLEAN pass); MED-001, MED-002, LOW-001, LOW-002 remain OPEN

**Precedent for open pass-60 findings:** MED-001 (Timestamp→Utf8 mapping undocumented),
MED-002 (non-AuditBuffer domains return data-empty rows — documented deferral gap),
LOW-001 (synthetic-slug fallback when org_registry present but slug_for returns None),
LOW-002 (AC-8 test does not include explain.rs/pushdown.rs in stub-residue check).

## Part A — Fix Verification (Pass-60 open findings)

Pass-60 MED/LOW findings were non-blocking observations; no fix burst was applied between
pass-60 and pass-61. Verification status for each:

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-W3MT-P60-MED-001 | MEDIUM | OPEN (no fix applied) | Timestamp→Utf8 mapping still undocumented in schema functions. Not a runtime defect; tracking for wave-5. |
| ADV-W3MT-P60-MED-002 | MEDIUM | OPEN (no fix applied) | Non-AuditBuffer domain deserialization still deferred; wildcard arm comment present. Acceptable pre-merge with TD annotation. |
| ADV-W3MT-P60-LOW-001 | LOW | OPEN (no fix applied) | Synthetic-slug fallback path still exists in resolve_source_refs ALL-scope branch (lines 540-558 of materialization.rs). |
| ADV-W3MT-P60-LOW-002 | LOW | OPEN (partially addressed) | AC-8 test still omits explain.rs and pushdown.rs. See NEW finding ADV-W3MT-P61-LOW-001 below for updated evidence. |

## Part B — New Findings

This pass independently reviewed the full diff and identified one genuinely new finding not
raised in any prior pass.

### CRITICAL

*No CRITICAL findings.*

### HIGH

*No HIGH findings.*

### MEDIUM

*No new MEDIUM findings.*

### LOW

#### ADV-W3MT-P61-LOW-001: `translate_push_down_filter` in `pushdown.rs` Contains `todo!()` in a `pub(crate)` Production Function — Invisible to AC-8 Stub-Residue Test

- **Severity:** LOW
- **Category:** coverage-gap
- **Location:** `crates/prism-query/src/pushdown.rs` — `translate_push_down_filter` (line 189)
- **Description:** `pushdown.rs` contains `pub(crate) fn translate_push_down_filter(_predicate, _columns) -> Option<String>` whose entire body is `todo!("S-3.X — sensor-specific filter translation")`. This is a `pub(crate)` production function, not a `#[cfg(test)]` function. It is not called from any production code path in the current diff (no callers in `engine.rs`, `materialization.rs`, or `explain.rs`), so it will not panic at runtime. However: (1) the AC-8 stub-residue test (`test_AC_8_no_todo_or_unimplemented_remains`) only checks `engine.rs`, `materialization.rs`, and `internal_tables.rs` — it does NOT check `pushdown.rs`. The `todo!()` is therefore invisible to the AC-8 guard. (2) The companion `ac-8-stub-residue-clean.log` is 0 bytes, so no `rg`-based workspace scan was recorded in the demo evidence. This means neither the inline test nor the log can prove that `pushdown.rs` is clean. If `translate_push_down_filter` is called by any future code before the `todo!()` is replaced, it will panic in production. POL-12 requires that all stub residue in production paths be tracked via TD annotation when deferral is acceptable.
- **Evidence:** `pushdown.rs` line 189: `pub(crate) fn translate_push_down_filter` body = `todo!("S-3.X — sensor-specific filter translation")`. `execute_integration_tests.rs` line 1347: `test_AC_8_no_todo_or_unimplemented_remains` `include_str!` list covers `engine.rs`, `materialization.rs`, `internal_tables.rs` — `pushdown.rs` is absent. `ac-8-stub-residue-clean.log` git blob `e69de29bb2d1d6434b8b29ae775ad8c2e48c5391` = 0 bytes (empty). No caller of `translate_push_down_filter` found in any production source file on the feature branch.
- **Proposed Fix:** Two options — the implementer should pick one: (a) Add `pushdown.rs` and `explain.rs` to the `include_str!` list in `test_AC_8_no_todo_or_unimplemented_remains`. Since `translate_push_down_filter` contains `todo!()`, the test would immediately fail — requiring either replacing the `todo!()` with a stub `None` return (correct for a deferred feature) or adding a POL-12-approved TD annotation comment and allowlisting. (b) Replace the `todo!()` body with `None` (the function returns `Option<String>`; returning `None` is semantically correct — "no sensor-specific filter translation available yet") and add a comment: `// TODO(TD-XXX): sensor-specific filter translation (S-3.X wave-5). Returns None = no pushdown; DataFusion evaluates post-scan.` The `None` return is safe — callers that receive `None` fall through to post-scan filter evaluation, which is the current behavior anyway. This eliminates the `todo!()` without behavior change and makes the function safe to call. Either option satisfies POL-12; option (b) is lower-effort and eliminates the latent panic risk entirely.

---

## Project Policy Rubric — Compliance Check

**POL-1 (append_only_numbering):** No evidence of renumbered or reused IDs in the PR diff. Existing IDs in `prism-core/src/error.rs` follow sequential numbering. COMPLIANT.

**POL-3 (state_manager_runs_last):** Not directly verifiable from diff alone; process governance. No violation observed.

**POL-10 (demo_evidence_story_scoped):** All demo evidence lives under `docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/` as a subfolder. No flat files at `docs/demo-evidence/*.md`. COMPLIANT.

**POL-12 (production_stub_residue_blocks_merge):** PARTIAL. `translate_push_down_filter` in `pushdown.rs` contains `todo!()`. The function has no callers in the current diff, so it is not reachable from a production execution path — however, it is a `pub(crate)` function in production source code (not `#[cfg(test)]`). Per POL-12 verification step 3, hits not inside `#[cfg(test)]` blocks are violations when story has `status: merged`. Story status is currently `draft`, so this does not block merge today. Before status flips to `merged`, this `todo!()` must be replaced with a non-panicking stub (`None`) or a TD annotation per the allowlist procedure. NOTED — non-blocking at `draft` status.

**POL-16 (no_inverted_polarity_tests_outside_red_gate):** No `#[should_panic(expected = ...)]` with stub-indicating messages found in the new test file. COMPLIANT.

**POL-18 (test_injection_feature_pairing):** `prism-query` has no `*_test_injection` Cargo feature. Not applicable. COMPLIANT.

**POL-14 (bc_vp_promotion_on_anchor_merge):** Story status is `draft` — no BC promotion expected yet. Post-merge: `behavioral_contracts: [BC-2.11.001, BC-2.11.005, BC-2.11.006, BC-2.11.007, BC-2.11.011, BC-2.11.012, BC-2.15.011]` must all promote to `active`. Noted as post-merge state-manager responsibility.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 0 |
| LOW | 1 |

**Overall Assessment:** pass-with-findings

**Convergence:** CLEAN PASS — no CRIT/HIGH findings. This is pass 4 (PR-level), contributing to convergence streak 2/3. The single LOW finding is a pre-existing gap (AC-8 scope omission + latent `todo!()` in uncalled function) that refines the earlier ADV-W3MT-P60-LOW-002 observation with concrete new evidence (the `translate_push_down_filter` todo! site). It does not block merge.

**Readiness:** The code is in merge-ready state for the current `draft` story status. The LOW finding (ADV-W3MT-P61-LOW-001) should be addressed before the story flips to `merged` per POL-12. The open MED findings from pass-60 should be tracked as TD stories for wave-5.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 61 (PR-level, fresh-context, Pass 4) |
| **New findings** | 1 (0 CRIT, 0 HIGH, 0 MED, 1 LOW) |
| **Duplicate/variant findings** | 0 (ADV-W3MT-P61-LOW-001 refines pass-60 LOW-002 with new concrete evidence — the specific `translate_push_down_filter` todo! site — rather than restating the same observation) |
| **Novelty score** | 1/1 = 1.0 (new evidence on a partially-overlapping gap) |
| **Median severity** | LOW |
| **Trajectory** | PR-P01: 13 findings (2C 5H 4M 2L) → PR-P02: 6 findings (1C 2H 1M 2L) → PR-P03: 4 findings (0C 0H 2M 2L) → PR-P04: 1 finding (0C 0H 0M 1L) |
| **Verdict** | FINDINGS_REMAIN (1 LOW, no CRIT/HIGH — counts as CLEAN pass per Standing Rule 3. Monotonically decreasing trajectory. Convergence streak 2/3 at PR level.) |
