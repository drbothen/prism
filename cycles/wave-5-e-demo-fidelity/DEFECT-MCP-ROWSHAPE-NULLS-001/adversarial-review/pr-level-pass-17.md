---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-MCP-ROWSHAPE-NULLS-001
passes: [17]
feature_head_at_review: 5d2624aa
date: 2026-07-14
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 2
  crit: 0
  high: 2
  med: 0
  low: 0
  obs: 0
  process_gap: 0
  out_of_scope_obs: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 17 — DEFECT-MCP-ROWSHAPE-NULLS-001

---

## Pass 17 (frozen 5d2624aa; fresh-context adversary; PR #222 MCP row-shape null serialization + [H8b] redundancy sweep + threatintel .prx staleness gate + retryable-coverage expansion + 28-explicit-VariantMeta-arms + 117-variant sentinel + ci.yml committed-.prx validation-before-build + security.rs fragment-hardened + BC-2.10.007 v1.19 CursorCapExceeded category "internal" + EC-11-081 NaN/±Inf→null locking test; PR-LEVEL cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

---

## Findings

### F-MCPRS-PRL17-HIGH-001 [HIGH][POL-23 pin-currency] — CLOSED @081dfbbc D-1756 burst (concurrency artifact)

**Severity:** HIGH
**Classification:** POL-23 pin-currency — BC-INDEX.md carried a stale BC-2.11.001 v1.21 head pin after the v1.21→v1.22 bump committed by fix-burst-27. At the moment the pass-17 adversary inspected `.factory/specs/behavioral-contracts/BC-INDEX.md`, the D-1756 burst had not yet committed; the stale v1.21 row was a live intra-state contradiction between the declared BC version and the index record.
**Status:** CLOSED @081dfbbc (D-1756 burst): BC-INDEX updated to v8.18 with the BC-2.11.001 v1.22 row. This is a concurrency artifact — the adversary observed the pre-commit state; the fix was already in-flight when the finding was recorded.

**Finding:** At frozen 5d2624aa, inspection of `.factory/specs/behavioral-contracts/BC-INDEX.md` found the BC-2.11.001 row still carrying version `v1.21` in the head-pin column. The STATE.md frontmatter at the time of inspection declared `bc_index_version: "8.17"` (the pre-D-1756 value); D-1756 was the burst that bumped both BC-2.11.001 to v1.22 and BC-INDEX to v8.18. Because the adversary conducted this pass against the `.factory/` pre-commit state (before D-1756's single-commit landed), it observed the old index row. The v1.22 bump codified EC-11-081 (non-finite Float64 → JSON null boundary artifact); a live stale pin misrepresents the canonical contract version to any agent reading only the index.

**Severity rationale:** HIGH because: (1) POL-23 mandates immediate index-row update on every BC version bump; (2) the stale v1.21 pin was live at the inspection timestamp; (3) any agent reading BC-INDEX to determine the canonical BC-2.11.001 version would receive the wrong version and potentially act on the old contract semantics.

**Closure evidence (D-1756 burst @081dfbbc):**

(1) **BC-INDEX v8.17→v8.18**: BC-2.11.001 row updated to `v1.22`. Residual grep for `v1.21` in BC-INDEX: zero hits on the BC-2.11.001 row. The concurrency gap is closed — any future read of BC-INDEX returns `v1.22` as the canonical pin.

(2) **Concurrency context**: The D-1756 burst was the authoritative fix for the v1.21→v1.22 transition. Pass-17 observed the pre-burst state; the finding is valid as a recorded observation but the closure was already committed before this pass record was authored.

---

### F-MCPRS-PRL17-HIGH-002 [HIGH][POL-23 pin-currency + content] — CLOSED in two parts

**Severity:** HIGH
**Classification:** POL-23 pin-currency + content gap — interface-definitions.md `§1.3 query tool JSON output schema` (`rows.items.description` field at line ~488) carried a stale `BC-2.11.001 v1.21` citation after the v1.22 bump AND lacked any mention of EC-11-081 (the Non-finite Float64 → JSON null boundary behavior codified in BC-2.11.001 v1.22). A caller reading interface-definitions.md alone would not know that Float64 NaN/±Infinity values serialize to JSON null (key present, indistinguishable from Arrow null at the wire boundary), creating a material wire-shape documentation gap at the primary MCP caller reference document.
**Status:** CLOSED in two parts: (1) stale v1.21 pin advanced to v1.22 @081dfbbc (D-1756 burst, interface-definitions v2.12→v2.13; same concurrency artifact as F-MCPRS-PRL17-HIGH-001); (2) EC-11-081 companion sentence + lane-attribution correction applied by PO (interface-definitions v2.13→v2.14, D-1757 burst).

**Finding:** At frozen 5d2624aa, inspection of `interface-definitions.md §1.3 query tool JSON output schema` found:

(1) **Stale v1.21 citation**: The `rows.items.description` field referenced `BC-2.11.001 v1.21` (pre-EC-11-081 version). After fix-burst-27 bumped BC-2.11.001 to v1.22, the interface-definitions.md inline pin was not updated in the same commit. The pin-currency gap violated POL-23: BC-INDEX declared v1.22 as canonical; interface-definitions.md still cited v1.21.

(2) **EC-11-081 content silence**: The `rows.items.description` at the EC-11-079 citation site described the null-not-absent invariant (every row carries a uniform key set; NULL cells as JSON null; missing key is a contract violation). No mention of EC-11-081: callers reading interface-definitions.md alone could not know that Float64 NaN/±Infinity values also serialize as JSON null (key present, value null, indistinguishable from Arrow null at the wire boundary). This creates a documentation gap for any client integrating with the `query` MCP tool that handles Float64 columns.

**Severity rationale:** HIGH because: (1) POL-23 is mandatory on every BC version bump; (2) the stale v1.21 pin at a primary caller-facing document (the MCP interface contract) contradicts the BC-INDEX canonical version; (3) EC-11-081 content absence means callers of the `query` tool have no documented wire-shape contract for non-finite Float64 values — they cannot write correct deserialization code from this document alone; (4) interface-definitions.md is the primary reference document for MCP callers; a content gap here propagates to all downstream integrators. The two issues are co-filed as one HIGH (same closure burst for the pin-currency part; EC-11-081 acknowledgment closes the content gap).

**Closure evidence (two parts):**

**(1) Pin-currency closure — D-1756 burst @081dfbbc (interface-definitions v2.12→v2.13; concurrency artifact):**

BC-2.11.001 inline citation in `rows.items.description` advanced from `v1.21` to `v1.22`. Residual grep for `v1.21` on the BC-2.11.001 citation in the file: zero hits after D-1756 burst. The combined EC-11-079 + EC-11-081 citation block is present; the v1.22 pin is now consistent with BC-INDEX v8.18.

**(2) EC-11-081 content gap + lane-attribution correction — D-1757 burst (interface-definitions v2.13→v2.14):**

(a) **EC-11-081 companion sentence added by PO**: `rows.items.description` at the EC-11-079 citation site now reads: "Non-finite Float64 values (NaN, ±Infinity) serialize as JSON null per EC-11-081 (BC-2.11.001 v1.22) — key present, indistinguishable from Arrow null at the wire boundary; callers MUST NOT rely on distinguishing NaN from missing data." Combined citation block updated to include `F-MCPRS-PRL17-HIGH-002 2026-07-14`. TD-VSDD-060 sweep: only one value-serialization description site in the file (line ~488); no other sites enumerate Float64/type-specific serialization semantics; zero additional sites requiring the same companion sentence.

(b) **Lane-attribution correction**: v2.14 changelog row Burst column corrected from `DEFECT-PQL-FNCALL-LHS-001 pass-17` (wrong lane) to `DEFECT-MCP-ROWSHAPE-NULLS-001 pass-17` (correct lane). The finding ID prefix `F-MCPRS-PRL17-HIGH-002` is unambiguously a DEFECT-MCP-ROWSHAPE-NULLS-001 PR-LEVEL finding; the PQL-lane attribution was a transcription error at v2.14 authoring.

---

## SAP-1 Emission Catalog Probe

**PASS.** Pass-17 fixes are spec-only (BC-INDEX pin update, interface-definitions.md companion sentence, changelog row correction). Zero net-new `event_type =` emissions introduced. No BC-2.16.002 catalog row required.

---

## Positive Verifications

- **EC-11-081 locking test confirmed load-bearing on production path (TD-VSDD-059):** `test_BC_2_11_001_EC_11_081_nonfinite_float_serializes_as_json_null` at @5d2624aa constructs a Float64 column containing NaN, +Inf, -Inf and asserts all three serialize to JSON null with key present; 481/481 prism-mcp GREEN. The test exercises the `server.rs` `WriterBuilder` production path — not a doc-only or rename closure.
- **CursorCapExceeded v1.19 wiring confirmed:** BC-2.10.007 v1.19 fields (category `"internal"`, `original_params_valid: true`, `retryable: false`, `ec_code_override: Some("E-STORE-020")`, suggestion text) confirmed present at @5d2624aa from fix-burst-25; unchanged at @5d2624aa.
- **Retryable whitelist mutation-resistant:** retryable whitelist logic in `prism-mcp/src/server.rs` uses an exhaustive match against the `RetryableCategory` enum — confirmed at @5d2624aa; the `CursorCapExceeded`/`"internal"` case is explicitly non-retryable (retryable: false) and cannot be silently changed by enum extension without compile-time failure.
- **Staleness mechanisms sound and correctly ordered:** CI committed-.prx validation step confirmed BEFORE build in `ci.yml` at @5d2624aa (fix-burst-25 closure); unchanged.
- **SAP-1 PASS:** `event_type =` emission sites at @5d2624aa sampled against BC-2.16.002 §Postconditions — all catalogued; no new emissions from pass-17 spec-only fixes.
- **Ingress-path non-finite-proof claims corroborated at code level:** 4 ingress paths (enrichment UDF `from_f64` → None; RFC 8259 JSON ingress; PQL grammar rejection; DataFusion div-by-zero → `PrismError::QueryExecutionFailed`) confirmed non-finite-proof at @5d2624aa; no reachable production path produces NaN/±Inf in a Float64 column today.

---

## Summary

**CLEAN(strict): NO** (2 HIGH — not zero-finding)
**CLEAN(PR-merge): NO** (2 HIGH findings — HIGH blocks both CLEAN(strict) and CLEAN(PR-merge) per BC-5.39.001)

Streak: **0/3** (no new push in this pass; streak remains at 0/3 from fix-burst-27 push @5d2624aa; DRIFT-ORCH-PRLEVEL-PUSH-001: only a new push resets; spec-only fixes without push do not reset the streak, but a non-CLEAN pass still cannot advance it)

Both findings CLOSED without a new push:
- **F-MCPRS-PRL17-HIGH-001 CLOSED (D-1756 burst @081dfbbc concurrency artifact):** BC-INDEX v8.17→v8.18; BC-2.11.001 v1.22 pin live; residual grep zero stale v1.21 hits.
- **F-MCPRS-PRL17-HIGH-002 CLOSED (two parts):** pin-currency D-1756 burst @081dfbbc (v2.13); EC-11-081 companion sentence + lane-attribution fix D-1757 burst PO (v2.14); TD-VSDD-060 zero additional sites; combined citation block correct.

CASCADE TALLY: 37 passes / 27 fix-bursts. Frozen HEAD @5d2624aa UNCHANGED (spec-only closures; no push); streak 0/3; next = PR-LEVEL pass 18 on same frozen 5d2624aa (spec-only fixes do not require a push; streak gate remains 0/3).
