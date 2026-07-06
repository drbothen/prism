---
document_type: adversarial-review
scope: LOCAL
pass: 1
story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
feature_head: fd379ede
fix_burst_head: 89a09782
date: 2026-07-06
clean_strict: false
clean_pr_merge: false
finding_counts: {HIGH: 1, MED: 2, LOW: 3, process_gap: 1, OBS: 2}
streak_after: 0/3
---

# LOCAL Adversary Pass 1 — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

**Pass result:** NOT CLEAN(strict), NOT CLEAN(PR-merge)
**Findings:** 1 HIGH + 2 MED + 3 LOW + 1 process-gap + 2 OBS
**Code HEAD at review:** fd379ede
**Fix-burst HEAD:** 89a09782
**Post-fix-burst just check:** GREEN 5206 tests; non-exhaustive 89/89
**LOCAL streak:** 0/3 (pass 1 not clean — streak resets)
**Next:** LOCAL pass 2 on frozen 89a09782

---

## SAP Probe Results

**SAP-1 (tracing emission catalog completeness):** PASS — `infusion.coercion_failed` catalog row present in BC-2.16.002 v1.94 (D-1550 pre-pass). No new event_type sites introduced without catalog row.

**SAP-2 (DTU↔TOML schema parity):** FLAG — cyberint sensor TOML declared `iocs_value_first` / `behaviors_ioc_value_first` columns but the e2e adapter test exercising the DTU roundtrip for these columns was missing. CLOSED via HIGH-001 fix @22e1b4a4.

---

## Findings and Closure Dispositions

### ADV-P01-HIGH-001 (HIGH) — SAP-2: Missing real e2e adapter test for cyberint surface columns

**Finding:** `iocs_value_first` and `behaviors_ioc_value_first` columns declared in cyberint TOML spec had no real e2e adapter test exercising the DTU array-extraction path. SAP-2 probe requires all TOML columns to have a corresponding DTU roundtrip or unit test.

**Closure:** Real e2e adapter test added + cyberint surface fix committed @22e1b4a4. SAP-2 FLAG → CLOSED.

---

### ADV-P01-MED-001 (MED) — InfusionError::TypeCoercionFailed not constructed or emitted

**Finding:** `InfusionError::TypeCoercionFailed` variant was defined but not constructed or emitted in any code path. The INV-ENRICH-TYPED-001 invariant (BC-2.19.001 §D6) requires this error to be emitted on type coercion failure; the catalog row in BC-2.16.002 (`infusion.coercion_failed`) requires an actual emission site.

**Closure:** `InfusionError::TypeCoercionFailed` constructed and emitted in the coercion failure path @d6c7a8ac.

---

### ADV-P01-MED-002 (MED) — declared_type field uses non-spec vocabulary at 5 sites

**Finding:** BC-2.16.002 spec uses the field name `declared_type` but the implementation used a different vocabulary form at 5 sites. Arrow-Debug format deviation also present.

**Closure:** `declared_type = output_type.as_str()` applied at all 5 sites; Arrow-Debug deviation removed @d6c7a8ac. BC-2.16.002 v1.94→v1.95 (product-owner spec-vocabulary reconciliation, D-1551).

---

### ADV-P01-LOW-001 (LOW) — E-INFUSE-013 sub-condition 7 message text missing "output_type"

**Finding:** Error taxonomy E-INFUSE-013 sub-condition 7 message template did not include `{field}="output_type"` as specified by error-taxonomy v2.16.

**Closure:** Sub-condition 7 message updated to include "output_type" @309e1975. error-taxonomy v2.15→v2.16 (product-owner, D-1551).

---

### ADV-P01-LOW-002 (LOW) — RGT-012/013/014 use non-structural TOML parse approach

**Finding:** Red Gate tests 012, 013, and 014 used non-structural parsing rather than the canonical TOML parse path, reducing test fidelity.

**Closure:** RGT-012/013/014 refactored to structural TOML-parse approach @2a775623.

---

### ADV-P01-LOW-003 (LOW) — AC-002 grep pattern too broad

**Finding:** AC-002 acceptance criterion grep pattern `output_type.*Utf8` would match sanctioned fallback expressions and guard comparisons, producing false positives.

**Closure:** AC-002 grep narrowed to `return_type.*Utf8` in story v1.1 (story-writer). Sanctioned `output_type.*Utf8` occurrences confirmed correct.

---

### PROCESS-GAP — column_type examples in PascalCase rather than lowercase canonical form

**Finding:** ADR-051 column_type examples used PascalCase (`String`, `Integer`, `Float`, `Boolean`, `Json`, `Datetime`) rather than the canonical lowercase snake_case form (`string`, `integer`, `float`, `boolean`, `json`, `datetime`) required by TOML serde deserialization.

**Closure:** ADR-051 v1.3→v1.4 column_type examples corrected to lowercase (architect, ARCH-INDEX v2.172→v2.173). AC-010 in story v1.1 corrected to lowercase (story-writer). D-1551.

---

### TD-VSDD-060 — Real duplicate parse_datetime_to_micros (orchestrator grep caught; adversary missed)

**Finding:** `parse_datetime_to_micros` exists in BOTH `prism-bin/spec_driven_adapter.rs` (ADR-052 original) AND `prism-spec-engine/src/datetime.rs` (new in this story). This is a real TD-VSDD-060 duplicate violation — not a sibling-site sweep issue. The adversary flagged it as a "known area for scrutiny" but did not classify it as a finding. The orchestrator independently verified via grep that it is a real duplicate and requires consolidation.

**Closure:** Real duplicate `parse_datetime_to_micros` consolidated in-scope @89a09782 (implementer fix, orchestrator-directed). One canonical implementation retained; all call sites updated.

---

### OBS-001 (OBS) — Non-blocking observation

**Closure:** Disposed non-blocking. No code or spec change required.

---

### OBS-002 (OBS) — Non-blocking observation

**Closure:** Disposed non-blocking. No code or spec change required.

---

## Fix-Burst Commit Chain

| SHA | Change |
|-----|--------|
| 22e1b4a4 | HIGH-001: real e2e adapter test + cyberint surface fix (SAP-2 closed) |
| d6c7a8ac | MED-001 + MED-002: TypeCoercionFailed emitted; declared_type vocabulary at 5 sites |
| 309e1975 | LOW-001: E-INFUSE-013 sub-cond 7 message "output_type" |
| 2a775623 | LOW-002: RGT-012/013/014 structural TOML-parse |
| 89a09782 | TD-VSDD-060: parse_datetime_to_micros duplicate consolidated (orchestrator-directed) |

Spec-only (no code SHA):
- LOW-003: AC-002 grep tightened — story v1.1 (story-writer)
- PROCESS-GAP: column_type lowercase — ADR-051 v1.4 (architect) + story v1.1 AC-010 (story-writer)
- MED-002 spec-side: BC-2.16.002 v1.94→v1.95 (product-owner)
