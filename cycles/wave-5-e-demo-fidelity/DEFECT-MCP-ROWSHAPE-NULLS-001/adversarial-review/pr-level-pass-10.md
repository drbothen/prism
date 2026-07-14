---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [10]
feature_head_at_review: fa0e4d70
date: 2026-07-14
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 3
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 10 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 10 (frozen fa0e4d70; fresh-context adversary; PR #222 MCP row-shape null serialization + H8b redundancy sweep + threatintel .prx staleness gate + retryable-coverage expansion; PR-LEVEL cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

---

## Findings

### F-MCPRS-PRL10-OBS-001 [OBS/supply-chain][Cargo.lock tracking] — CLOSED (human-ratified 2026-07-14 + fix-burst-22 sidecar @ac5bf335 @6b2a7c8e)

**Severity:** OBS (supply-chain observation)
**Classification:** supply-chain — Cargo.lock tracking status and plugin version re-cut
**BC:** N/A (workspace hygiene / supply-chain discipline)

**Finding:** The threatintel-lookup plugin crate at `crates/prism-spec-engine/plugins/threatintel-lookup/` was at version 1.0.1. Cargo.lock reflects the resolved dependency tree at that version. The adversary observed that plugin-crate Cargo.lock tracking policy was not explicitly documented in the project, raising a supply-chain audit question: (a) is Cargo.lock committed for the plugin crate? and (b) does the version number need incrementing when the .prx binary is re-generated after a build artifact change?

**Impact:** Supply-chain audit gap — if Cargo.lock is not tracked, reproducible builds for the plugin crate cannot be verified. Severity: OBS (structural; not a runtime defect; not blocking PR merge per the `.prx` staleness gate already in place from prior fix-bursts).

**Resolution (human-ratified 2026-07-14 + fix-burst-22 @ac5bf335, @6b2a7c8e):**
- **Human ratification:** Cargo.lock IS tracked in the workspace (confirmed 2026-07-14). The supply-chain concern is resolved by existing tracking policy.
- **Plugin version bump:** threatintel-lookup plugin re-cut at 1.0.2 with updated .prx build artifact. Sidecar commit @ac5bf335 documents the version bump rationale.
- **Final commit @6b2a7c8e:** Workspace Cargo.lock updated to reflect plugin 1.0.1→1.0.2 bump; non-exhaustive 91/91; prism-mcp 474/474; just check 5511/5511 GREEN. LOCAL-ONLY (3 commits over origin fa0e4d70; push deferred to pass-11 gate).

---

### F-MCPRS-PRL10-OBS-002 [OBS/test-fragility][/dev/null fixture seam] — CLOSED (fix-burst-22 @b4f3485e)

**Severity:** OBS (test-fragility)
**Classification:** test-fragility — platform-specific /dev/null fixture dependency
**BC:** N/A (test infrastructure hygiene)

**Finding:** A test fixture in the prism-mcp test suite used `/dev/null` as a mock endpoint URL to exercise the error path when the MCP server cannot connect. The `/dev/null` path is a POSIX-ism: it resolves as a valid filesystem path on macOS/Linux but does not exist on Windows, producing a different error variant (`io::Error(NotFound)` vs `io::Error(PermissionDenied)` vs `reqwest::Error` depending on platform). The test asserted a specific error variant that would only hold on the CI Linux runner. On Windows contributors' machines, the test would either fail or produce a false-positive pass via a different error code path.

**Impact:** Zero runtime impact. The gap is test portability — the test would not produce the same result across all supported platforms. Severity: OBS (test fragility; CI passes because CI is Linux; Windows dev machines would see breakage).

**Resolution (fix-burst-22 @b4f3485e):** Implementer replaced the `/dev/null` fixture seam with an in-process mock server bound to `127.0.0.1:0` (ephemeral port, OS-assigned) that immediately closes the connection. The mock server is cross-platform and hermetic. The test now asserts the correct connection-refused error variant across macOS, Linux, and Windows. prism-mcp suite GREEN after fix. just check incremental GREEN.

---

### F-MCPRS-PRL10-OBS-003 [OBS/non-exhaustive][VariantMeta catch-all arm] — CLOSED (fix-burst-22 @6dee4036 + PO spec @4b50ce8b)

**Severity:** OBS (non-exhaustive match)
**Classification:** non-exhaustive — VariantMeta match arm uses `_ => {}` catch-all instead of explicit enumeration
**BC:** BC-2.10.007 (§RETRYABLE-503 VariantMeta categorization; updated v1.17→v1.18 by this closure)

**Finding:** The MCP server's `VariantMeta` match in the error-categorization path used a single `_ => {}` wildcard arm to handle all non-retryable response variants. The codebase carries a `#[non_exhaustive]` gate policy (91 types enforced, EXPECTED=91 in `check-non-exhaustive.sh`). While `VariantMeta` itself is not in the non-exhaustive gate (it is an internal match, not a public API surface type), the wildcard catch-all means that adding a new `VariantMeta` variant in a future burst could silently bypass the categorization without a compile-time warning. Additionally, the BC-2.10.007 §Postconditions for retryable categorization did not document the full enumeration, making it hard for a future implementer to verify completeness.

**Impact:** Not a current runtime defect — all existing VariantMeta variants are handled correctly. The risk is forward-maintenance: new variants added silently fall through. Severity: OBS (maintenance-fragility).

**Resolution (fix-burst-22 @6dee4036 + PO spec @4b50ce8b):**
- **Code @6dee4036:** Implementer replaced the `_ => {}` catch-all with 28 explicit VariantMeta arms covering the full current enumeration. Added a compile-time sentinel: a `const _: ()` assertion that the VariantMeta variant count is exactly 117, firing a compile error if a new variant is added without updating the match. `just check` 5511/5511 GREEN; non-exhaustive 91/91; prism-mcp 474/474.
- **Spec @4b50ce8b (PO):** BC-2.10.007 v1.17→v1.18 — §Postconditions updated to document the explicit-arm enumeration truth: 28 explicit arms + 117-variant sentinel. BC-INDEX v8.14→v8.15 (state-manager D-1751 sweep).

---

## SAP-1 Emission Catalog Probe

**PASS.** No new `event_type =` emissions introduced by the branch relative to develop@5f1b5771. All emissions present in the branch were catalogued in BC-2.16.002 §Postconditions in prior bursts.

---

## Summary

**CLEAN(strict): NO** (3 OBS findings — supply-chain tracking gap, fixture /dev/null seam, VariantMeta catch-all)
**CLEAN(PR-merge): YES** (zero CRIT/HIGH/MED findings; 3 OBS are non-blocking)

Streak: **0/3** (stays 0/3; OBS findings prevent strict-clean advancement per BC-5.39.001).

All 3 OBS findings closed by fix-burst-22: OBS-001 human-ratified + sidecar @ac5bf335 + Cargo.lock @6b2a7c8e; OBS-002 @b4f3485e (cross-platform mock seam); OBS-003 @6dee4036 (28 explicit arms + 117-variant sentinel) + PO spec @4b50ce8b (BC-2.10.007 v1.17→v1.18). Branch final HEAD: @6b2a7c8e (3 LOCAL-ONLY commits over origin fa0e4d70; push pending at pass-11 gate). Cascade tally: 30 passes / 22 fix-bursts. Streak 0/3 on @6b2a7c8e (push from fix-burst-22 resets per DRIFT-ORCH-PRLEVEL-PUSH-001 when executed). PR-LEVEL pass 11 dispatched on frozen @6b2a7c8e after push.
