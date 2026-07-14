---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [13]
feature_head_at_review: c82f30ba
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 1
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 13 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 13 (frozen c82f30ba; fresh-context adversary; PR #222 MCP row-shape null serialization + [H8b] redundancy sweep + threatintel .prx staleness gate + retryable-coverage expansion + 28-explicit-VariantMeta-arms + 117-variant sentinel + ci.yml sidecar diagnostic + scripts/hash-plugin-source.py repo_root anchoring; PR-LEVEL cascade; streak 1/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

---

## Findings

### F-MCPRS-PRL13-MED-001 [MED][POL-4 semantic-anchoring/doc-drift] — CLOSED @01f6070c (fix-burst-24: 3 comments corrected + proactive full 117-arm audit)

**Severity:** MED
**Classification:** POL-4 semantic-anchoring / doc-drift — 117-variant sentinel arm comment maintenance contract violated; stale category comments in `error_category_coverage.rs`
**Status:** CLOSED — fix-burst-24 @01f6070c

**Finding:** The 117-variant sentinel in `crates/prism-core/src/tests/error_category_coverage.rs` carries an explicit docstring maintenance contract: each arm's inline comment records the MCP structured-error category that `error_mapping.rs` produces for that variant — the comments are intended to be "kept in sync with BC-2.10.007 v1.18 §Category table." Two arms carry stale comments that contradict BC-2.10.007 v1.18 §Category and the production `error_mapping.rs` match:

**Stale comment 1 — `SensorTimeout` arm (line 51):** Comment reads `"transient"`. The actual category produced by `error_mapping.rs` for `SensorTimeout` is `"upstream_error"` (with `retryable: true`). `SensorTimeout` is classified as a retryable upstream error — the upstream sensor did not respond in time. The `"transient"` category label belongs exclusively to `SensorRateLimited` (which correctly has its own arm and carries the `"transient"` comment). The `"transient"` category is a distinct error classification from `"upstream_error"` in the BC-2.10.007 taxonomy; the maintenance contract comment is wrong.

**Stale comment 2 — `SensorHttpError` arm (line 50):** Comment reads `"503→transient; 429→transient"`. The actual category for `SensorHttpError` remains `"upstream_error"` in all cases — per BC-2.10.007 §RETRYABLE-503 adjudication, 429 and 503 responses from a sensor trigger `retryable: true` in the structured error, but the category field stays `"upstream_error"`. `SensorRateLimited` (a separate, dedicated variant for the 429 rate-limited path) is the sole route to `"transient"` category. The comment falsely implies category transitions with HTTP status codes, which contradicts the BC-2.10.007 single-category-per-variant design.

**Severity rationale:** MED because the 117-variant sentinel's per-arm comment carries an explicit maintenance contract — it is not decorative prose. A developer auditing `error_category_coverage.rs` against BC-2.10.007 §Category to understand what categories the MCP structured-error schema can emit would find two arms with wrong categories, undermining the sentinel's function as a living documentation anchor. If a future BC amendment reclassifies `SensorTimeout`, the existing stale comment increases the probability of the propagation sweep in the test file being skipped (the stale annotation already exists, so the "sync needed" signal is already in a wrong state). The finding is MED rather than HIGH because the production runtime behavior is correct — `error_mapping.rs` produces the right categories; only the documentation contract in the test file is wrong.

**Fix plan — fix-burst-24:** Correct both arm comments to match BC-2.10.007 v1.18 §Category. Conduct a proactive full 117-arm audit of all arm comments against BC-2.10.007 §Category table to catch any additional stale comments before the cascade advances. TD-VSDD-060 sweep: grep for `SensorTimeout.*transient` and `SensorHttpError.*transient` across all `crates/` Rust files to confirm no additional stale claims at other sites.

**Closure evidence @01f6070c (fix-burst-24):**

(1) `SensorTimeout` arm (line 51): corrected from `"transient"` → `"upstream_error (retryable: true per BC-2.10.007 §RETRYABLE-503)"`. Category now correctly reflects the `error_mapping.rs` arm output and BC-2.10.007 §Category classification.

(2) `SensorHttpError` arm (line 50): corrected from `"503→transient; 429→transient"` → `"upstream_error (retryable=true for 408|425|429|500|502|503|504 per BC-2.10.007 §RETRYABLE-503)"`. Category stays `"upstream_error"` for all HTTP status code paths; retryable flag annotation added per §RETRYABLE-503.

(3) **Proactive full 117-arm audit catch:** During fix-burst-24, the implementer audited all 117 arm comments against BC-2.10.007 §Category. This audit found a **third stale comment not caught by pass-13:**

- `SensorAuthFailed` arm (approximately line 47): comment read `"authentication"`. Actual category in BC-2.10.007 §Category and `error_mapping.rs` is `"permission"` — authentication failures at sensor adapters map to the `"permission"` structured-error category in the MCP schema (the user's credentials for the sensor are not recognized; this is a permissions issue from the MCP client's perspective, not a generic authentication flow event). `"authentication"` is not a valid category value in the BC-2.10.007 taxonomy. Corrected to `"permission"` in the same fix-burst-24 commit @01f6070c.

**TD-VSDD-060 sweeps:**
- `SensorTimeout.*transient`: 1 hit (the corrected line) — no other stale occurrences.
- `SensorHttpError.*transient`: 5 hits — 1 fixed (the stale parenthetical on the arm comment); 4 remaining hits are legitimate references in comments/docs that correctly describe the `retryable` boolean behavior (not the category field); verified non-normative.
- Post-fix full 117-arm audit: 114 remaining arms all correct against BC-2.10.007 §Category; zero additional stale category claims.

**Test verification @01f6070c:** `just iter prism-core` 261/261 GREEN. `just check-fast` clean.

---

## SAP-1 Emission Catalog Probe

**PASS.** Approximately 85 `crates/` `event_type =` emission sites sampled at HEAD c82f30ba against BC-2.16.002 §Postconditions Canonical Structured Event Catalog — all catalogued. No new `event_type` emissions introduced by branch relative to develop@5f1b5771.

---

## Positive Verifications

- **EC-11-079 single `with_explicit_nulls(true)` chokepoint:** `server.rs` WriterBuilder construction confirmed as sole WriterBuilder site in `prism-mcp/src/`; null-not-absent contract enforced at one gated location.
- **Red-gate finder predicate topology:** `.is_none_or(|v| v.is_null())` predicate correctly identifies JSON fields that are absent (`.is_none()`) OR explicitly null (`.is_null()`); topology matches the null-not-absent invariant — a correct field is present with an explicit null value, not absent. Walk-observable structure: pass requires that the null column is present in the response JSON with value `null`, not missing entirely.
- **Retryable whitelist boundary lock:** `matches!(status.as_u16(), 408|425|429|500|502|503|504)` 7-element whitelist in `error_mapping.rs` confirmed; 501 (Not Implemented) absent from whitelist (permanent error — correct); BC-2.10.007 §RETRYABLE-503 adjudication honored.
- **CI staleness gate reachability:** `build-plugin-threatintel-infusion` staleness gate CI step confirmed present and reachable; two-case structured diagnostic (case-a plugin not rebuilt / case-b byte-identical rebuild) verified YAML-valid under the ancestor-check condition structure.
- **`strip_url_to_host_port` RFC-3986 walk:** URL normalization function verified against RFC-3986 host+port extraction; no scheme-injection or path-traversal vectors in the stripping logic.
- **SAP-1 PASS (~85 sites):** All ~85 `event_type =` emission sites in `crates/` at HEAD c82f30ba verified against BC-2.16.002 §Postconditions Canonical Structured Event Catalog.
- **Manifest parity byte-identical:** Plugin manifest file contents verified byte-identical between committed `.prx` sidecar and manifest embedded in built plugin binary; no drift.

---

## Summary

**CLEAN(strict): NO** (1 MED — F-MCPRS-PRL13-MED-001 sentinel comment drift)
**CLEAN(PR-merge): NO** (1 MED finding — MED is non-blocking for PR-merge gate per BC-5.39.001 §CLEAN(PR-merge) definition, but CLEAN(strict) requires zero findings of any severity; streak cannot advance)

Streak: **0/3** (RESET from 1/3; MED finding prevents CLEAN(strict) advancement per BC-5.39.001).

F-MCPRS-PRL13-MED-001 CLOSED @01f6070c (fix-burst-24): 3 stale arm comments corrected — `SensorTimeout` (line ~51), `SensorHttpError` (line ~50), `SensorAuthFailed` (line ~47, proactive catch); TD-VSDD-060 sweeps clean; 261/261 prism-core GREEN; check-fast clean. NEW MCP HEAD: @01f6070c (LOCAL-ONLY; push pending).

CASCADE TALLY: 33 passes / 24 fix-bursts. PR-LEVEL pass 14 dispatched on frozen @01f6070c (streak 0/3).
