---
document_type: adversarial-review
scope: LOCAL
pass: 2
story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
feature_head: 89a09782
fix_burst_head: 4699551e
date: 2026-07-06
clean_strict: false
clean_pr_merge: false
finding_counts: {MED: 1, LOW: 2, OBS: 1, process_gap: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 2 — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

**Pass result:** NOT CLEAN(strict), NOT CLEAN(PR-merge)
**Findings:** 1 MED + 2 LOW + 1 OBS + 1 process-gap
**Code HEAD at review:** 89a09782
**Fix-burst HEAD:** 4699551e
**Post-fix-burst just check:** GREEN 5211 tests; non-exhaustive 89/89
**LOCAL streak:** 0/3 (pass 2 not clean — streak resets)
**Next:** LOCAL pass 3 on frozen 4699551e

---

## SAP Probe Results

**SAP-1 (tracing emission catalog completeness):** PASS — `infusion.coercion_failed` catalog row confirmed present in BC-2.16.002 v1.95. No new event_type sites introduced without catalog row. All tracing emission sites in crates/ verified covered.

**SAP-2 (DTU↔TOML schema parity):** PASS — pass-1 HIGH-001 closure (real e2e adapter test for cyberint surface columns) verified held. All TOML-declared columns have DTU-equivalent fields confirmed.

**TD-VSDD-059 (paper-fix detection):** PASS — `InfusionError::TypeCoercionFailed` is constructed and emitted in the coercion failure path AND asserted in tests (not a doc-comment or rename-only fix).

**TD-VSDD-060 (sibling-site sweep):** PASS — single canonical `parse_datetime_to_micros` implementation confirmed; pass-1 consolidation at 89a09782 held.

**ADR-051 D1–D6 conformance:** PASS — all six ADR-051 decisions (output_arrow_type(), coerce_to_typed(), E-INFUSE-013 sub-cond 7/8, TypeCoercionFailed, declared_type field, datetime=Timestamp(µs,UTC)) confirmed in implementation.

---

## Findings and Closure Dispositions

### ADV-P02-MED-001 (MED) — value-materialization test gap: types tested but no positive VALUE asserted

**Finding:** The coerce_to_typed positive-path tests (RGTs testing integer/float/boolean/datetime coercion success) verified that the coercion did NOT error and returned a non-null Arrow array, but no test asserted the materialized VALUE of the output cell. A test that only checks `is_ok()` or array length does not prove that the output bytes are correct — a silent truncation, rounding, or encoding error would pass. BC-2.19.001 §INV-ENRICH-TYPED-001 requires correct typed output, which requires asserting the actual value.

Additionally, RGT-002..005 (round-trip positive-path tests added in fix-burst-1) did not assert `.value(0)` on the resulting typed column — they confirmed array shape but not the materialized scalar value.

**Closure:** @4699551e — four new positive-value unit tests RGT-017..020 added to coerce_to_typed: integer→42, float→3.14, boolean→true, datetime→microseconds epoch value asserted via `.value(0)`. RGT-002..005 each augmented with `.value(0)` assertion confirming the materialized scalar. Load-bearing assertions in place; silent value corruption would cause test failure.

---

### ADV-P02-LOW-001 (LOW) — AC-011 real chain test: hand-fed value, not TOML-sourced

**Finding:** AC-011 acceptance criterion required "e2e chain test reads source_path from TOML spec, calls generate_with_scenario_iocs, asserts materialized column value." The test added in fix-burst-1 exercised the coercion logic but hand-fed the input value rather than reading `source_path` from the TOML file and invoking `generate_with_scenario_iocs()`. This means the integration path (TOML spec → generator → adapter → coerced output) was not fully exercised end-to-end.

**Closure:** @4699551e — test revised to: (1) parse the real TOML spec file to extract `source_path` for the relevant column, (2) call `generate_with_scenario_iocs()` with that source path, (3) assert the materialized typed column value from the generator output. The full TOML-source chain is now exercised as AC-011 requires.

---

### ADV-P02-LOW-002 (LOW) — story BC-2.16.002 pin v1.93 vs canonical v1.95

**Finding:** Story v1.1 (written by story-writer in fix-burst-1) cited BC-2.16.002 at v1.93 in several inline version-pin anchors within acceptance criteria. The canonical BC-2.16.002 version is v1.95 (bumped during fix-burst-1 spec reconcile: D-1551). The story's internal version citations lagged by 2 minor versions, creating citation drift.

**Closure:** Story v1.2 (story-writer) — all BC-2.16.002 version pins updated to v1.95. STORY-INDEX v2.592→v2.593.

---

### ADV-P02-OBS-001 (OBS) — redundant crowdstrike top-level behaviors_ioc_value_first

**Finding:** The crowdstrike scenario generator emitted a top-level `behaviors_ioc_value_first` field that duplicated the nested `behaviors[].ioc_value_first` field. The top-level field was not declared in the crowdstrike TOML sensor spec as a column and was not consumed by any AC or test. The redundant field added noise to the 50-record fixture set and created a silent spec-vs-generator drift.

Note: `cyberint_iocs.ioc_value_first` and `cyberint_iocs.behaviors_ioc_value_first` are KEPT in the cyberint generator — these map to a future `cyberint_iocs` table (documented in the generator comment). Only the crowdstrike top-level duplicate was removed.

**Closure:** @2167cbea — `behaviors_ioc_value_first` removed from crowdstrike scenario generator top-level emission and from the 50 committed fixture records. Generator comment added explaining the cyberint distinction.

---

### ADV-P02-OBS-002 [process-gap] — ci.yml prose "88" vs gate EXPECTED=89

**Finding:** `ci.yml` contained a prose comment referencing "88 non-exhaustive types" from a prior session's gate count. The active gate was already `EXPECTED=89` (bumped by TemporalLiteralPosition in S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 at D-1545). The prose comment and the gate value were out of sync — not a functional regression but a maintenance hazard that could cause confusion during future gate bumps.

**Closure:** @68124763 — ci.yml prose comment updated 88→89. Gate `EXPECTED=89` unchanged (correct value).

---

## Positive Verifications (pass-1 closures held)

All pass-1 findings confirmed closed and stable at 89a09782:

| Pass-1 Finding | Verification at 89a09782 | Status |
|----------------|--------------------------|--------|
| HIGH-001 (SAP-2 cyberint surface) | Real e2e adapter test present and passing | HELD |
| MED-001 (TypeCoercionFailed emitted) | TypeCoercionFailed constructed+emitted+asserted in tests (TD-VSDD-059) | HELD |
| MED-002 (declared_type=output_type.as_str()) | All 5 sites confirmed correct; Arrow-Debug deviation removed | HELD |
| LOW-001 (E-INFUSE-013 sub-cond 7 message) | Sub-condition 7 message includes "output_type" | HELD |
| LOW-002 (RGT-012/013/014 structural TOML-parse) | Structural parse approach confirmed in all three | HELD |
| LOW-003 (AC-002 grep tightened) | Story v1.1 AC-002 grep uses narrow pattern | HELD |
| process-gap (ADR-051 column_type lowercase) | ADR-051 v1.4 examples all lowercase; AC-010 corrected | HELD |
| TD-VSDD-060 (parse_datetime_to_micros dedup) | Single canonical implementation in prism-spec-engine/src/datetime.rs | HELD |
| OBS-001/002 (disposed non-blocking) | No follow-up required | HELD |

---

## Fix-Burst-2 Commit Chain

| SHA | Change |
|-----|--------|
| 68124763 | ADV-P02-OBS-002 [process-gap]: ci.yml prose 88→89 (gate EXPECTED=89 unchanged) |
| 2167cbea | ADV-P02-OBS-001: remove crowdstrike behaviors_ioc_value_first from generator + 50 fixture records |
| 4699551e | ADV-P02-MED-001 + ADV-P02-LOW-001: coerce_to_typed positive-value tests RGT-017..020 + RGT-002..005 .value(0) assertions; AC-011 real TOML-source chain test |

Spec-only (no code SHA):
- ADV-P02-LOW-002: BC-2.16.002 pins v1.93→v1.95 — story v1.2 (story-writer); STORY-INDEX v2.592→v2.593
