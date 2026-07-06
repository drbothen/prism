---
document_type: adversarial-review
scope: LOCAL
pass: 3
story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
feature_head: 4699551e
fix_burst_head: ce93229a
date: 2026-07-06
clean_strict: false
clean_pr_merge: true
finding_counts: {LOW: 1, OBS: 3}
streak_after: 0/3
---

# LOCAL Adversary Pass 3 — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

**Pass result:** CLEAN(PR-merge)=yes, NOT CLEAN(strict) (1 LOW + 3 OBS)
**Findings:** 1 LOW + 3 OBS
**Code HEAD at review:** 4699551e
**Fix-burst HEAD:** ce93229a
**Post-fix-burst just check:** GREEN 5212 tests; non-exhaustive 89/89
**LOCAL streak:** 0/3 (pass 3 not strict-clean — streak resets)
**Next:** LOCAL pass 4 on frozen ce93229a

---

## SAP Probe Results

**SAP-1 (tracing emission catalog completeness):** PASS — `infusion.coercion_failed` catalog row confirmed present in BC-2.16.002 v1.95. No new event_type sites introduced without catalog row. All tracing emission sites in crates/ verified covered.

**SAP-2 (DTU↔TOML schema parity):** PASS — crowdstrike redundant field removal (pass-2 OBS-001 closure) clean. All TOML-declared columns have DTU-equivalent fields confirmed. cyberint_alerts column unaffected — uses nested $.iocs[0].value on alert records.

**TD-VSDD-059 (paper-fix detection):** PASS — `InfusionError::TypeCoercionFailed` is constructed and emitted in the coercion failure path AND asserted in tests (closure held from pass-1/2).

**TD-VSDD-060 (sibling-site sweep):** PASS — single canonical `parse_datetime_to_micros` implementation confirmed; pass-1 consolidation held through ce93229a.

**ADR-051 D1–D6 conformance:** PASS — all six ADR-051 decisions (output_arrow_type(), coerce_to_typed(), E-INFUSE-013 sub-cond 7/8, TypeCoercionFailed, declared_type field, datetime=Timestamp(µs,UTC)) confirmed in implementation.

**Positive-value assertions:** PASS — RGT-017..020 assert materialized scalar values (.value(0)); AC-011 chain loads TOML source_path and invokes generate_with_scenario_iocs (closures from pass-2 held).

---

## Findings and Closure Dispositions

### ADV-P03-LOW-001 (LOW) — misleading Int64 "42.0" fallback comment + untested EC-002 path

**Finding:** The `coerce_to_typed` implementation contained a comment for the Int64 arm reading approximately "if the string contains a decimal (e.g. '42.0'), parse as f64 then cast" — describing a float-then-cast fallback strategy. This comment was misleading: the actual EC-002 contract (float-string input when output type is Integer) requires returning `None` (type mismatch yields null), not attempting a float-to-integer cast. Additionally, no test exercised the `coerce_to_typed("95.7", Int64)` path to confirm the correct `None` return.

**Closure:** @27d4da21 — comment corrected to accurately describe the `None` return for float-string input on Integer type. `test_ec002_float_string_to_integer_yields_null` added: asserts `coerce_to_typed("95.7", Int64)` returns `None` as EC-002 requires. Load-bearing assertion in place.

---

### ADV-P03-OBS-002 (OBS) — EC-002 and EC-006 paths untested

**Finding:** Edge case EC-002 (float-string input for Integer output type → null) and EC-006 (empty string input → null) were documented in the story but had no corresponding Red Gate tests driving them. A test gap for declared contract edge cases is a SOUL.md #4 (silent untested path) concern even at OBS severity.

**Closure:** @27d4da21 — `test_ec002_float_string_to_integer_yields_null` and `test_ec006_empty_input_yields_null` added to the Red Gate table (RGT-021 and RGT-022 respectively). Story updated v1.2→v1.3 with EC-002/EC-006 rows referencing the new RGTs; red_gate_tests 20→22. STORY-INDEX v2.593→v2.594.

---

### ADV-P03-OBS-001 (OBS) — speculative cyberint top-level iocs_value_first field

**Finding:** A `iocs_value_first` field was emitted at the top level of the cyberint scenario generator output. Independent inspection confirmed this field was not declared as a column in the cyberint TOML sensor spec and was not consumed by any AC or test. The field appeared to be a speculative pre-computed scalar duplicating the nested `$.iocs[0].value` extraction path.

Note: `cyberint_iocs.ioc_value_first` and `cyberint_iocs.behaviors_ioc_value_first` fields serving the future `cyberint_iocs` table WERE retained — those map to a distinct future table (documented in generator comment). Only the speculative top-level duplicate was removed.

**Closure:** @ce93229a — consumer audit confirmed no live reader of the top-level `iocs_value_first` field. Field removed from cyberint scenario generator. The `cyberint_alerts` column's `iocs_value_first` data originates from the nested JSONPath `$.iocs[0].value` extraction in the spec-driven adapter — unaffected.

---

### ADV-P03-OBS-003 (OBS) — story resources.rs File Structure claim inaccurate

**Finding:** The story `resources.rs File Structure` section listed a task describing sensor-specific column names (e.g., `iocs_value_first`) as if the implementation would directly introduce those names into the resources.rs module. The actual implementation uses GENERIC placeholders (`sensor_table`, `src_ip`, etc.) per the F-PQL2/CRIT-001 genericization decision. The story text created a false impression of sensor-specific resource bindings in that file.

**Closure:** Story v1.3 (story-writer) — `resources.rs File Structure` task annotated to clarify that the file uses generic placeholders per the F-PQL2/CRIT-001 genericization; no sensor-specific column (e.g., `iocs_value_first`) appears in resources.rs. No code change — the generic placeholder implementation is correct.

---

## Positive Verifications (all prior closures independently re-derived GREEN)

| Finding | Verification at 4699551e | Status |
|---------|--------------------------|--------|
| pass-1 HIGH-001 (SAP-2 cyberint surface) | Real e2e adapter test present and passing | HELD |
| pass-1 MED-001 (TypeCoercionFailed emitted) | TypeCoercionFailed constructed+emitted+asserted (TD-VSDD-059) | HELD |
| pass-1 MED-002 (declared_type=output_type.as_str()) | All 5 sites confirmed correct | HELD |
| pass-1 LOW-001 (sub-cond7 message) | Sub-condition 7 message includes "output_type" | HELD |
| pass-1 LOW-002 (RGT-012/013/014 structural TOML-parse) | Structural parse approach confirmed | HELD |
| pass-1 LOW-003 (AC-002 grep tightened) | Narrow grep pattern confirmed in story v1.2+ | HELD |
| pass-1 process-gap (ADR-051 column_type lowercase) | ADR-051 v1.4 examples all lowercase | HELD |
| pass-1 TD-VSDD-060 (parse_datetime_to_micros dedup) | Single canonical implementation in prism-spec-engine/src/datetime.rs | HELD |
| pass-2 MED-001 (positive-value assertions RGT-017..020) | .value(0) assertions confirmed in coerce_to_typed tests | HELD |
| pass-2 LOW-001 (AC-011 real TOML-source chain) | generate_with_scenario_iocs called with source_path from TOML | HELD |
| pass-2 LOW-002 (BC-2.16.002 pin v1.95) | All story citations at v1.95 | HELD |
| pass-2 OBS-001 (crowdstrike behaviors_ioc_value_first removed) | Top-level field absent from crowdstrike generator + fixtures | HELD |
| pass-2 OBS-002/process-gap (ci.yml prose 88→89) | ci.yml EXPECTED=89 + prose comment consistent | HELD |

---

## Fix-Burst-3 Commit Chain

| SHA | Change |
|-----|--------|
| 27d4da21 | ADV-P03-LOW-001 + ADV-P03-OBS-002: comment corrected; test_ec002_float_string_to_integer_yields_null + test_ec006_empty_input_yields_null (RGT-021/022) added |
| ce93229a | ADV-P03-OBS-001: remove speculative cyberint top-level iocs_value_first field (cyberint_alerts column unaffected; consumer audit confirmed no live reader) |

Spec-only (no code SHA):
- ADV-P03-OBS-003: resources.rs File Structure annotated — story v1.3 (story-writer); STORY-INDEX v2.593→v2.594
