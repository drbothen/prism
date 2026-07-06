---
document_type: adversarial-review
scope: LOCAL
passes: [4, 5, 6]
story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
feature_head: ce93229a
fix_burst_head: a45573e4
date: 2026-07-06
clean_strict: false
clean_pr_merge: true
finding_counts: {LOW: 1, OBS: 2}
streak_after: 0/3
---

# LOCAL Adversary Passes 4 + 5 + 6 — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

---

## Pass 4 (frozen ce93229a)

**Pass result:** CLEAN(strict)=yes, CLEAN(PR-merge)=yes — zero findings any severity
**Code HEAD at review:** ce93229a
**LOCAL streak after pass-4:** 1/3 (advanced from 0/3)
**Note:** Pass-4 was the first CLEAN(strict) pass on ce93229a. All prior closures
re-verified GREEN. SAP-1/2 PASS, TD-VSDD-059/060 PASS, ADR-051 D1-D6 conformant,
positive-value assertions PASS, result-correctness PASS.

---

## Pass 5 (frozen ce93229a)

**Pass result:** CLEAN(PR-merge)=yes, NOT CLEAN(strict) (1 LOW + 2 OBS)
**Findings:** 1 LOW + 2 OBS
**Code HEAD at review:** ce93229a
**Fix-burst HEAD:** a45573e4
**Post-fix-burst just check:** GREEN 5213 tests; non-exhaustive 89/89
**LOCAL streak after pass-5:** 0/3 (RESET from 1/3 — pass-5 not strict-clean)
**Next:** LOCAL pass 7 on frozen a45573e4 (fresh sequential 3-CLEAN attempt)

---

## Pass 6 (frozen ce93229a)

**Pass result:** CLEAN(strict)=yes — **VOID for streak purposes**
**Code HEAD at review:** ce93229a (same HEAD as pass-5)
**Streak impact:** NONE — this pass does NOT advance the streak
**Void rationale:** Pass-6 was run in PARALLEL with pass-5 on the same HEAD ce93229a.
Pass-5 proved that HEAD is NOT strictly clean. Per the frozen-HEAD consecutive-streak rule
(DRIFT-ORCH-PRLEVEL-PUSH-001 analog applied to LOCAL cascades), a CLEAN pass on a HEAD
concurrently proven non-clean does not advance the streak. The streak can only advance on
passes taken AFTER the fix-burst that addressed the non-clean finding, against the updated
HEAD.

**Process lesson recorded:** Run LOCAL convergence passes SEQUENTIALLY, not in parallel,
once findings are possible. Running parallel passes on the same HEAD is valid only if the
goal is independent auditing of the same snapshot (e.g., dual-adversary coverage), not for
streak advancement. Parallel-clean passes on a non-clean HEAD create ambiguous audit trails.

---

## SAP Probe Results (Pass 5, verified against ce93229a)

**SAP-1 (tracing emission catalog completeness):** PASS — no new event_type sites without
BC-2.16.002 catalog row. `infusion.coercion_failed` catalog row confirmed present at v1.95.
All tracing emission sites in crates/ verified covered.

**SAP-2 (DTU↔TOML schema parity):** PASS — cyberint/crowdstrike TOML-declared columns
all have DTU-equivalent fields. No new TOML columns introduced without DTU counterpart.

**TD-VSDD-059 (paper-fix detection):** PASS — `InfusionError::TypeCoercionFailed` is
constructed, emitted, and asserted in tests (pass-1/2/3 closure held through ce93229a).

**TD-VSDD-060 (sibling-site sweep):** PASS — single canonical `parse_datetime_to_micros`
implementation confirmed (pass-1 consolidation held).

**ADR-051 D1–D6 conformance:** PASS — all six decisions (output_arrow_type(), coerce_to_typed(),
E-INFUSE-013 sub-cond 7/8, TypeCoercionFailed, declared_type field, datetime=Timestamp(µs,UTC))
confirmed in implementation at ce93229a.

**Positive-value assertions:** PASS — RGT-017..022 all assert materialized scalar values.
AC-011 TOML-source chain confirmed load-bearing.

**Result-correctness:** PASS — float `10.0 >= 8.0` numeric comparison; datetime coercion
to Timestamp(Microsecond, Some("UTC")) confirmed.

---

## Findings and Closure Dispositions (Pass 5)

### ADV-P05-LOW-001 (LOW) — story §References error-taxonomy pin v2.15 vs canonical v2.16 (POL-25)

**Finding:** The story's §References section cited `error-taxonomy.md v2.15` while the canonical
error-taxonomy version recorded in STATE.md frontmatter and the actual artifact is v2.16 (bumped
by S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 which added E-QUERY-042). POL-25 (version-pin discipline)
requires spec citations to track the canonical version. The mismatch creates false confidence that
the story was spec-reviewed against the current error taxonomy.

**Closure:** Story v1.3→v1.4 (story-writer) — §References error-taxonomy pin corrected v2.15→v2.16.
No behavioral change; the story's error contract surface (E-INFUSE-013, E-INFUSE-014) is unchanged
in v2.16. STORY-INDEX v2.594→v2.595.

---

### ADV-P05-OBS-001 (OBS) — AC-011 tests use manual JSON indexing, do not exercise production extract_at_path resolver

**Finding:** The AC-011 acceptance criterion tests (and the cyberint scenario used by them) drove
the JSONPath `$.iocs[0].value` extraction using manual array indexing in test fixtures rather than
routing through the production `extract_at_path` helper function. The production code path for
numeric-index JSONPath resolution (`$.array[N].field`) was therefore not regression-guarded —
a future refactor of `extract_at_path` numeric-index handling could silently break
`$.iocs[0].value` retrieval without failing any test.

**Closure:** @2fbaadff (implementer) — `test_extract_at_path_numeric_index_resolves_first_element`
added to `prism-spec-engine` tests. The test asserts that `extract_at_path("$.iocs[0].value", ...)`
on a JSON value containing `iocs: [{value: "1.2.3.4"}]` returns `"1.2.3.4"`, directly
regression-guarding the numeric-index resolution path that `$.iocs[0].value` and
`$.behaviors[0].ioc_value` rely on.

---

### ADV-P05-OBS-002 (OBS) — stale stub-phase todo!()/Red-Gate-holds doc-comments in implemented output_arrow_type/coerce_to_typed + RGT-007..010 doc-comments

**Finding:** Several doc-comments in `output_arrow_type` and `coerce_to_typed` contained
stub-phase language: `todo!()` references, "Red Gate holds" notes, and RGT-007..010 annotations
describing the pre-implementation state. These comments were accurate during the stub phase but
became stale after implementation was complete. Stale doc-comments create confusion about whether
a code path is actually implemented or deferred.

**Closure:** @a45573e4 (implementer) — stub-phase `todo!()`/Red-Gate-holds doc-comments
removed from `output_arrow_type` and `coerce_to_typed`. RGT-007..010 doc-comment annotations
updated to reflect that these are now implemented (not stub) paths. No behavioral change.

---

## Positive Verifications (all prior closures independently re-derived GREEN at ce93229a)

| Finding | Verification at ce93229a | Status |
|---------|--------------------------|--------|
| pass-1 HIGH-001 (SAP-2 cyberint surface) | Real e2e adapter test present and passing | HELD |
| pass-1 MED-001 (TypeCoercionFailed emitted) | TypeCoercionFailed constructed+emitted+asserted (TD-VSDD-059) | HELD |
| pass-1 MED-002 (declared_type=output_type.as_str()) | All 5 sites confirmed correct | HELD |
| pass-1 LOW-001 (sub-cond7 message) | Sub-condition 7 message includes "output_type" | HELD |
| pass-1 LOW-002 (RGT-012/013/014 structural TOML-parse) | Structural parse approach confirmed | HELD |
| pass-1 LOW-003 (AC-002 grep tightened) | Narrow grep pattern confirmed in story v1.4+ | HELD |
| pass-1 process-gap (ADR-051 column_type lowercase) | ADR-051 v1.4 examples all lowercase | HELD |
| pass-1 TD-VSDD-060 (parse_datetime_to_micros dedup) | Single canonical implementation in prism-spec-engine/src/datetime.rs | HELD |
| pass-2 MED-001 (positive-value assertions RGT-017..020) | .value(0) assertions confirmed in coerce_to_typed tests | HELD |
| pass-2 LOW-001 (AC-011 real TOML-source chain) | generate_with_scenario_iocs called with source_path from TOML | HELD |
| pass-2 LOW-002 (BC-2.16.002 pin v1.95) | All story citations at v1.95 | HELD |
| pass-2 OBS-001 (crowdstrike behaviors_ioc_value_first removed) | Top-level field absent from crowdstrike generator + fixtures | HELD |
| pass-2 OBS-002/process-gap (ci.yml prose 88→89) | ci.yml EXPECTED=89 + prose comment consistent | HELD |
| pass-3 LOW-001 (EC-002 comment + test) | Comment accurate; test_ec002_float_string_to_integer_yields_null present | HELD |
| pass-3 OBS-002 (RGT-021/022 EC-002/EC-006) | test_ec002 + test_ec006_empty_input_yields_null present; red_gate_tests=22 | HELD |
| pass-3 OBS-001 (cyberint speculative iocs_value_first removed) | Top-level field absent from cyberint generator | HELD |
| pass-3 OBS-003 (resources.rs annotation) | Generic placeholder annotation present in story v1.4 | HELD |

---

## Fix-Burst-4 Commit Chain

| SHA | Change |
|-----|--------|
| 2fbaadff | ADV-P05-OBS-001: add test_extract_at_path_numeric_index_resolves_first_element in prism-spec-engine (regression-guards $.iocs[0].value / $.behaviors[0].ioc_value numeric-index resolution) |
| a45573e4 | ADV-P05-OBS-002: remove/update stale stub-phase todo!()/Red-Gate-holds doc-comments in output_arrow_type/coerce_to_typed + RGT-007..010 doc-comment annotations |

Spec-only (no code SHA):
- ADV-P05-LOW-001: story §References error-taxonomy pin v2.15→v2.16 — story v1.4 (story-writer); STORY-INDEX v2.594→v2.595
