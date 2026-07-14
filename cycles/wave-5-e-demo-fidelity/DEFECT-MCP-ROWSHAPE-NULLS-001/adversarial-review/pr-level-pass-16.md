---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [16]
feature_head_at_review: 9e116a01
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 2
  crit: 0
  high: 1
  med: 0
  low: 1
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 16 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 16 (frozen 9e116a01; fresh-context adversary; PR #222 MCP row-shape null serialization + [H8b] redundancy sweep + threatintel .prx staleness gate + retryable-coverage expansion + 28-explicit-VariantMeta-arms + 117-variant sentinel + ci.yml committed-.prx validation-before-build + security.rs fragment-hardened + BC-2.10.007 v1.19 CursorCapExceeded category "internal" + error-taxonomy v2.51; PR-LEVEL cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

---

## Findings

### F-MCPRS-PRL16-HIGH-001 [HIGH][POL-23 version-pin propagation gap] — CLOSED fix-burst-27 (story-writer: S-TEST-WIRESHAPE-SWEEP-001 v0.18→v0.19 + S-MCP-E003 v0.8→v0.9)

**Severity:** HIGH
**Classification:** POL-23 version-pin propagation gap — BC-2.10.007 v1.18→v1.19 bump (fix-burst-25) shipped without a story-pin sweep, leaving 13 stale `v1.18` live sites across S-TEST-WIRESHAPE-SWEEP-001 v0.18 and 10 stale sites across S-MCP-E003 v0.8 (9 stale `v1.18` + 1 stale `v1.13` BC-table cell from the initial draft missed by every prior sweep). STATE.md already declared the BC-INDEX at v1.19 — the exact mismatch POL-23 is designed to prevent.
**Status:** CLOSED fix-burst-27 (story-writer: S-TEST-WIRESHAPE-SWEEP-001 v0.18→v0.19 + S-MCP-E003 v0.8→v0.9; residual grep zero live hits)

**Finding:** State inspection at frozen 9e116a01 found:

(1) **S-TEST-WIRESHAPE-SWEEP-001 v0.18** (the primary POL-23-obligated wire-shape story): 13 live `v1.18` pin sites present. BC-INDEX was at v8.17 (reflecting v1.19 as canonical) — a direct intra-state contradiction. BC-2.10.007 v1.19 introduced CursorCapExceeded category `"internal"` / `original_params_valid: true` / `retryable: false` with a specific MCP error-response shape; story pin sites that still cite v1.18 implicitly endorse the old `"validation"` category semantics (wrong).

(2) **S-MCP-E003-SERIALIZATION-MIGRATION-001 v0.8** (sibling story sharing the same BC): 9 live `v1.18` pin sites + 1 live `v1.13` BC-table cell in the initial draft that had been missed by every prior sweep since D-1729 story registration. The stale `v1.13` cell is a compounding error — v1.13 predates the category-field addition entirely.

**Severity rationale:** HIGH because: (1) POL-23 is a mandatory sweep obligation triggered on every BC version bump; (2) fix-burst-25 (BC-2.10.007 v1.18→v1.19) was the bump event; (3) STATE.md explicitly declared BC-INDEX at v8.17 reflecting v1.19 — the story files contradicted this live recorded state; (4) two downstream stories carried stale pins, creating spec-chain inconsistency visible to any agent or reviewer reading those stories. The S-7.01 partial-fix propagation pattern: the pin-sweep obligation was known (it fires after every BC bump per POL-23), but was not fulfilled during fix-burst-25 itself.

**Fix plan — fix-burst-27:** story-writer sweeps and corrects all stale pin sites: S-TEST-WIRESHAPE-SWEEP-001 v0.18→v0.19 (13 sites), S-MCP-E003 v0.8→v0.9 (10 sites: 9 stale v1.18 + 1 stale v1.13 BC-table cell). Residual grep verifies zero live stale pins after the sweep.

**Closure evidence (fix-burst-27):**

(1) **S-TEST-WIRESHAPE-SWEEP-001 v0.19**: 13 stale `BC-2.10.007 v1.18` sites updated to `v1.19`. All occurrences now reflect the correct CursorCapExceeded category "internal" semantics. Residual grep for `v1.18` across the story file: zero hits.

(2) **S-MCP-E003-SERIALIZATION-MIGRATION-001 v0.9**: 9 stale `v1.18` sites + 1 stale `v1.13` BC-table cell corrected to `v1.19`. The `v1.13` cell was in the story's BC-table initial draft and had survived undetected through 5 prior sweep passes. Residual grep for `v1.18` and `v1.13` across the story file: zero live stale hits.

(3) **POL-23 obligation fulfilled**: STORY-INDEX updated to record both story version bumps (STORY-INDEX v2.682→v2.683).

---

### F-MCPRS-PRL16-LOW-001 [LOW][untested-edge/spec-silence] — CLOSED fix-burst-27 (PO: BC-2.11.001 v1.21→v1.22 EC-11-081 + locking test @5d2624aa PUSHED)

**Severity:** LOW
**Classification:** untested-edge / spec-silence — Float64 NaN / +Inf / -Inf values through the arrow-json serialization path were untested and contract-silent. Two plausible behaviors existed (JSON null vs. serialization error), neither of which had been codified in BC-2.11.001 or error-taxonomy.

**Status:** CLOSED fix-burst-27 via empirical probe → PO codification → locking test: probe confirmed arrow-json 58.2.0 HARDCODES non-finite → JSON null (no builder option; key present; EC-11-079 key-presence contract holds; behavior indistinguishable from Arrow null at the schema level); all 4 current ingress paths are non-finite-proof (UDF `from_f64` → None; RFC 8259 JSON boundary; PQL grammar; DataFusion div-by-zero errors). PO ratified Option A (codify boundary artifact; Option B scan declined — cost without reachable benefit): BC-2.11.001 v1.21→v1.22, new EC-11-081 (collision-checked, namespace high-water EC-11-080). Locking test `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null` landed @5d2624aa (481/481 prism-mcp). POL-23 follow-up: S-TEST-WIRESHAPE-SWEEP-001 v0.19→v0.20 (8 v1.21→v1.22 pin sites).

**Finding:** At frozen 9e116a01, the arrow-json serialization path for Float64 columns carried no contract for non-finite values (NaN, +Inf, -Inf). Inspection:

- BC-2.11.001 §Postconditions: null-not-absent row-shape invariant (EC-11-079) covers NULL Arrow values → JSON null. No mention of non-finite Float64 behavior.
- error-taxonomy.md: no E-QUERY entry for non-finite serialization failure.
- prism-mcp tests: no test exercising a Float64 column containing NaN or ±Inf.

Two behaviors were a priori plausible:
- **(A)** arrow-json serializes non-finite as JSON null (key present, value `null`) — indistinguishable from Arrow null to the client.
- **(B)** arrow-json returns a serialization error for non-finite floats — would require E-QUERY error path.

The absence of a contract or test meant that either behavior could change across arrow-json version bumps without detection.

**Severity rationale:** LOW because: (1) there is no currently reachable code path that inserts NaN/±Inf into a Float64 column used by prism-mcp (all ingress paths are non-finite-proof: UDF `from_f64` → None, RFC 8259 JSON boundary, PQL grammar rejection, DataFusion div-by-zero error path); (2) the gap is spec-silence, not active misbehavior; (3) however, the absence of a locking test means any future ingress path addition, arrow-json version bump, or UDF output change could silently alter the behavior. The boundary behavior must be codified before the PR merges to prevent future regressions from going undetected.

**Empirical probe (fix-burst-27 pre-codification):** Implementer injected a Float64 NaN value into a test column bypassing UDF normalization and confirmed: arrow-json 58.2.0 emits the column key with `null` value — no serialization error. The `with_explicit_nulls(true)` flag does not affect non-finite behavior; it is governed by `arrow_json`'s internal IEEE 754 non-finite handling which maps non-finite to JSON null unconditionally (no builder option to change this). The key IS present; EC-11-079 holds for non-finite floats by this mechanism.

**PO adjudication — Option A:** Codify the arrow-json boundary artifact as a documented invariant: non-finite Float64 values serialize to JSON null (key present). This is pragmatic codification of the arrow library constraint, not a desired design choice. Option B (proactive NaN gate at query execution boundary) was declined: no reachable ingress path currently; adding a plan-time scan adds cost without current benefit; a future ingress addition should add its own gate at that boundary.

**Closure evidence (fix-burst-27):**

(1) **BC-2.11.001 v1.22**: EC-11-081 allocated — "Non-finite Float64 values (NaN, +Inf, -Inf) serialize as JSON `null` (key present; indistinguishable from Arrow null). Boundary artifact of `arrow_json` 58.2.0; no builder option governs this." EC-11-081 collision-checked against EC-11-080 (prior namespace high-water); no conflict. §Postconditions updated to reference EC-11-081 alongside EC-11-079.

(2) **Locking test** `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null` (fix-burst-27, @5d2624aa): constructs a Float64 column containing NaN, +Inf, -Inf; asserts all three serialize to JSON `null` with key present; asserts no serialization error. 481/481 prism-mcp GREEN at @5d2624aa. PUSHED to origin/fix/DEFECT-MCP-ROWSHAPE-NULLS-001 (PR #222 head updated to 5d2624aa by this push).

(3) **Ingress-path audit**: 4 paths verified non-finite-proof: (a) enrichment UDF `from_f64` → None (Arrow null, never non-finite); (b) RFC 8259 JSON ingress parses `nan`/`inf` as JSON parse errors before reaching the column; (c) PQL grammar rejects non-numeric tokens before DataFusion execution; (d) DataFusion div-by-zero → `PrismError::QueryExecutionFailed` (error path, never a non-finite column value). No reachable ingress produces NaN/±Inf in a Float64 column today; locking test exercises the behavior via test-controlled injection.

(4) **POL-23 sweep (v1.21→v1.22)**: S-TEST-WIRESHAPE-SWEEP-001 v0.19→v0.20 — 8 live `v1.21` BC-2.11.001 pin sites updated to `v1.22`. Residual grep: zero live stale `v1.21` pins.

---

## SAP-1 Emission Catalog Probe

**PASS.** `crates/` `event_type =` emission sites at HEAD 9e116a01 sampled against BC-2.16.002 §Postconditions Canonical Structured Event Catalog — all catalogued. Fix-burst-27 changes (BC-2.11.001 v1.21→v1.22 spec prose + story pin sweeps + locking test) introduced zero net-new `event_type =` emissions in production code. No BC-2.16.002 catalog row required.

---

## Positive Verifications

- **EC-11-079 single `with_explicit_nulls(true)` chokepoint:** `server.rs` `WriterBuilder` construction confirmed as sole `WriterBuilder` site in `prism-mcp/src/`; null-not-absent contract enforced at one gated location; unchanged from prior passes.
- **481/481 prism-mcp at @5d2624aa:** fix-burst-27 added exactly 1 net-new test (`test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null`); all prior 480 prism-mcp tests GREEN (no regressions).
- **BC-2.10.007 v1.19 CursorCapExceeded category fields:** category `"internal"`, `original_params_valid: true`, `retryable: false`, `ec_code_override: Some("E-STORE-020")`, suggestion `"Cursor capacity exhausted. Wait for existing cursors to close before retrying."` — confirmed present at 9e116a01 from fix-burst-25 closure; unchanged.
- **28-arm explicit VariantMeta groups:** unchanged at 9e116a01 from fix-burst-22 closure.
- **117-variant sentinel coverage:** unchanged at 9e116a01.
- **error-taxonomy v2.51 sibling-convention sweep:** `E-STORE-020` Description `(map_prism_error -32000 INTERNAL_ERROR)` annotation confirmed present (fix-burst-26 closure); sibling-convention compliant; unchanged.
- **5 Red Gate tests (null-not-absent):** confirmed present and GREEN at 9e116a01 (unchanged from prior passes).
- **EC-11-081 collision check:** EC-11-080 is the prior high-water mark (confirmed in BC-2.11.019 v1.18); EC-11-081 is unallocated → no conflict.

---

## Summary

**CLEAN(strict): NO** (1 HIGH + 1 LOW — not zero-finding)
**CLEAN(PR-merge): NO** (1 HIGH finding — HIGH blocks both CLEAN(strict) and CLEAN(PR-merge) per BC-5.39.001)

Streak: **0/3 RESET** (fix-burst-27 pushed new commit @5d2624aa to origin; DRIFT-ORCH-PRLEVEL-PUSH-001: any push to fix branch during cascade resets streak to 0/3).

Both findings CLOSED via fix-burst-27:
- **F-MCPRS-PRL16-HIGH-001 CLOSED (fix-burst-27 story-writer):** S-TEST-WIRESHAPE-SWEEP-001 v0.18→v0.19 (13 stale v1.18 sites corrected); S-MCP-E003-SERIALIZATION-MIGRATION-001 v0.8→v0.9 (9 stale v1.18 + 1 stale v1.13 BC-table cell corrected; 10 sites total); residual greps zero live stale pins; STORY-INDEX v2.682→v2.683.
- **F-MCPRS-PRL16-LOW-001 CLOSED (fix-burst-27 PO+implementer):** BC-2.11.001 v1.21→v1.22 EC-11-081 (non-finite Float64 → JSON null codified; probe verified; PO Option A ratified; 4 ingress paths non-finite-proof); locking test `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null` @5d2624aa (481/481 prism-mcp GREEN); PUSHED; streak RESET 0/3; POL-23 S-TEST-WIRESHAPE-SWEEP-001 v0.19→v0.20 (8 sites); BC-INDEX v8.17→v8.18; interface-definitions.md v2.12→v2.13 (BC-2.11.001 v1.22 pin).

CASCADE TALLY: 36 passes / 27 fix-bursts. NEW HEAD @5d2624aa PUSHED (PR #222 head updated); streak RESET 0/3 on FROZEN HEAD 5d2624aa; next = PR-LEVEL pass 17 on frozen 5d2624aa (streak 0/3).
