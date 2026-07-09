---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [2]
feature_head_at_review: 39c8b134
date: 2026-07-09
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 2 — FIX-IEQ-ERRPATH-001

---

## Pass 2 (frozen 39c8b134; fresh-context adversary; PR-LEVEL cascade; streak candidate 1/3 — ADVANCING — 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** 0 total (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**STREAK:** 1/3 — CLEAN(strict) on frozen 39c8b134. Streak advances to 1/3 per BC-5.39.001. Next pass 3 on SAME frozen HEAD required for 2/3.

**Code HEAD at review:** 39c8b134 (frozen; 7e23a2c2 + 39c8b134 on top of dacb60fa; pushed to origin; PR #219 OPEN base develop; 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** YES — ZERO findings of any severity (0 CRIT / 0 HIGH / 0 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**CLEAN(PR-merge):** YES — ZERO CRIT + HIGH + MED findings

---

## Findings

None.

---

## Probe Summary (15 probes, all empty-handed)

1. **Chokepoint sanitization line-by-line** — `ColumnNotFoundDetails::new` in `prism-core/src/error.rs` applies `sanitize_for_log` on construction; column-only field; `#[non_exhaustive]` + zero struct-literal sites outside the constructor = bypass-proof. No unguarded construction path exists.

2. **Levenshtein raw-vs-sanitized consistency** — For legitimate column identifiers (ASCII alphanumeric + underscore), `sanitize_for_log` is identity-preserving; Levenshtein distance computation and suggestion generation are unaffected. Adversarially-crafted inputs with control characters are out-of-contract at the `ColumnNotFoundDetails::new` boundary; suggestion quality for such inputs is not a correctness obligation.

3. **Double-sanitize idempotency** — `sanitize_for_log` applied at (a) the `column_not_found.rejected` tracing emission sites (original 51f071ff path) and (b) `ColumnNotFoundDetails::new` constructor (39c8b134 path). Both layers are independently load-bearing: (a) protects the log path from CWE-117 log injection; (b) protects the MCP-facing payload path from CWE-116 prompt injection. Applying both is defense-in-depth, not redundancy; each guards a distinct sink.

4. **MED-002 emission-path locks TD-VSDD-059 load-bearing** — The three `#[tracing_test::traced_test]` tests added at 7e23a2c2 exercise actual production code paths: (i) `check_query_column_availability` single-tenant path, (ii) `check_query_column_availability` multi-tenant path, (iii) `check_pipe_stage_columns` binding-context path. Each asserts `logs_contain("column_not_found.rejected")` AND `!logs_contain("\x01")`. These are not helper-unit-test-style assertions; they call production functions and observe the tracing output.

5. **OBS-001 payload gates verified on actual PrismError variant** — The two RED payload-injection gates added at 7e23a2c2 assert that `PrismError::ColumnNotFound { .. }` → `error_mapping.rs` → `ErrorData.data` does not contain control characters from the column name field. Verified these tests call the real `map_error_to_mcp` production path, not a stub.

6. **Spec-layer sync** — BC-2.11.016 v1.22 §Postconditions "Injection-safety of `column` (MCP-facing payload)" clause is present and accurate. BC-2.16.002 v2.08 catalog row `column_not_found.rejected` has the `sanitize_for_log` annotation on the `column` field, matching the `infusion.coercion_failed` sibling row. error-taxonomy v2.35 carries the {column} injection-safety clause for E-QUERY-038. Sibling pins v1.10 (BC-2.11.017) / v1.15 (BC-2.11.020) / v1.27 (BC-2.11.004) are current. 4 carrier stories v2.40 (S-DEMO-FIDELITY-REMEDIATION-001) / v2.17 (S-DEMO-PRISMQL-ONBOARDING-001-B) / v1.26 (S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001) / v1.51 (S-PRISMQL-CASE-INSENSITIVE-001) are current. POL-23/25/29 satisfied.

7. **BC "propagates" language = outcome description** — BC-2.11.016 v1.22 says injection-safety "propagates" through the error path; verified this is an outcome description (the safety is guaranteed by the chokepoint constructor), not a claim that each downstream function independently sanitizes. No code-spec mismatch.

8. **EC-11-076 + STAGE-JOIN spot-checks** — EC-11-076 (per-reference HEAD-JOIN scoping) regression locks are GREEN at 39c8b134. STAGE-JOIN suspension rule tests are GREEN. No regression from the pass-1 fix commits.

9. **Audit-script G-inventory** — scripts/t13-preflight-audit.py G-section items G1–G8 verified consistent with the code changes at 39c8b134: G2/G3/G6/G7/G8 are FAIL-on-error (correct for demo environment); G4 uses canonical anchor (not heuristic predicate); G5 WARN with justification. No G-item was inadvertently reverted.

10. **Materialization clippy fix behavior-preserving** — `materialization.rs:761` `&format!()` → `format!()` (Rust 1.97.0 `clippy::useless_borrows_in_formatting`). Verified the change is purely syntactic; the formatted string value is identical. Sibling sites in `materialization.rs` do not have the same redundant-borrow pattern (sweep intentional).

11. **Infusion match→? equivalence** — `dacb60fa` converted 3 `match` arms to `?` in `load_spec` / `load_spec_with_runtime` / `hot_reload` in `prism-spec-engine/src/infusion/mod.rs`. Verified all three conversions are semantically equivalent: the original `match` arms returned `Err(e)` on failure (same as `?`); no early-return path was altered; sibling sites in the same file were already using `?`.

12. **POL-24 byte-verbatim** — BC title strings in the 4 carrier stories are byte-verbatim matches against their source BCs. No title drift introduced in the pin round.

13. **mut-ast binding legitimate per ADR-052 §D4** — The `let mut ast = ...` binding in the temporal typing pass is required because the AST walk mutates the `ast` value in-place; the `mut` is not superfluous. ADR-052 §D4 Option A lenient-parse + AST-walk pattern explicitly requires mutable traversal. No clippy lint triggered on this binding under Rust 1.97.0.

14. **EC-11-076 boundary: qualified vs bare references** — Spot-checked that qualified references (column names of the form `table.column`) are not incorrectly suspended by the HEAD-JOIN rule. The BC-2.11.016 v1.21 per-reference scoping (segments.len()==2 qualified refs retain full E-QUERY-038 gate) is correctly implemented at 39c8b134.

15. **SAP-1 completeness re-verification** — Ran conceptual grep of all `event_type =` sites in changed files. No new `event_type` value introduced by the pass-1 fix commits (7e23a2c2 + 39c8b134). The only emission of `column_not_found.rejected` is the existing 3 sites already cataloged in BC-2.16.002 v2.08 row 177.

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset per DRIFT-ORCH-PRLEVEL-PUSH-001) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN)**

**Novelty:** LOW — All 15 probes returned empty-handed. The feature code (IEQ/IIN/INE column availability gate, injection-safety chokepoint, spec-layer sync) was fully validated across LOCAL passes 1–19 and PR-LEVEL pass 1. The two post-pass-1-fix commits (7e23a2c2 + 39c8b134) closed all pass-1 findings; the resulting code surface has no residual attack angles.

**Pattern:** Pass-2 adversary with fresh context performed 15 targeted probes covering the security-critical surfaces (chokepoint bypass, payload injection, spec-layer drift, emission catalog completeness, regression locks) and found zero issues. This is the characteristic pattern of a clean fix burst that addressed real findings without introducing secondary defects.

**Streak status:** 1/3 — CLEAN(strict) on frozen 39c8b134. **NEXT: PR-LEVEL adversary pass 3 on SAME frozen HEAD 39c8b134** (streak candidate 2/3; BC-5.39.001). No push to branch before pass 3 completes per DRIFT-ORCH-PRLEVEL-PUSH-001.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — BC-2.16.002 v2.08 catalog row `column_not_found.rejected` carries `sanitize_for_log` annotation on the `column` field, matching the sibling `infusion.coercion_failed` row. No new `event_type` values introduced in the pass-1 fix commits.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — 3 `#[tracing_test::traced_test]` emission-path locks (single-tenant, multi-tenant, binding-context) added at 7e23a2c2 exercise actual production code paths. MED-002 finding from pass-1 is load-bearing-closed; no residual paper-fix.

**TD-VSDD-060 (sibling-site sweep):** PASS — `ColumnNotFoundDetails::new` is the single construction chokepoint; TD-VSDD-060 sweep at 39c8b134 confirmed all callsites (prism-mcp, prism-query test callsites) use string literals for column names, not user-controlled input. No unswept sibling site.

**CWE-116 / AD-017 (injection-safety, MCP-facing payload):** PASS — `sanitize_for_log` applied at `ColumnNotFoundDetails::new` constructor in `prism-core/src/error.rs`; 2 RED payload-injection gates GREEN at 7e23a2c2; OBS-001 from pass-1 is fully closed.

**POL-14 (BC auto-promotion):** N/A — no story merge in this pass.

**BC-5.39.001 (3-CLEAN streak):** ADVANCED — pass-2 result CLEAN(strict) on frozen 39c8b134. Streak 0/3 → **1/3**. Two consecutive CLEAN(strict) passes on unchanged frozen 39c8b134 required to reach 3/3 convergence.
