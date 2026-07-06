---
document_type: adversarial-review
scope: LOCAL
passes: [7]
story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
feature_head: a45573e4
fix_burst_head: a3083468
date: 2026-07-06
clean_strict: false
clean_pr_merge: true
finding_counts: {LOW: 2, OBS: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 7 — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

---

## Pass 7 (frozen a45573e4)

**Pass result:** CLEAN(PR-merge)=yes, NOT CLEAN(strict) (2 LOW + 1 OBS)
**Findings:** 2 LOW + 1 OBS
**Code HEAD at review:** a45573e4
**Fix-burst HEAD:** a3083468
**Post-fix-burst just check:** GREEN 5213 tests; non-exhaustive 89/89
**LOCAL streak after pass-7:** 0/3 (UNCHANGED — pass-7 not strict-clean)
**Next:** LOCAL passes 8/9/10 SEQUENTIAL on frozen a3083468

**Finding class:** All three findings are the same recurring stale-stub-doc class as
pass-5 ADV-P05-OBS-002. The root cause is that fix-burst-4 swept stub-phase language
from the adversary-cited sites in `output_arrow_type` and `coerce_to_typed` but did not
perform an exhaustive grep across the entire diff for ALL instances of the same class.
This is a TD-VSDD-060 sibling-site sweep miss applied to doc-comments / S-7.01 (defensive
sweep discipline).

---

## SAP Probe Results (Pass 7, verified against a45573e4)

**SAP-1 (tracing emission catalog completeness):** PASS — no new event_type sites without
BC-2.16.002 catalog row. `infusion.coercion_failed` catalog row confirmed present at v1.95.
All tracing emission sites in crates/ verified covered.

**SAP-2 (DTU↔TOML schema parity):** PASS — cyberint/crowdstrike TOML-declared columns
all have DTU-equivalent fields. No new TOML columns introduced without DTU counterpart.

**TD-VSDD-059 (paper-fix detection):** PASS — `InfusionError::TypeCoercionFailed` is
constructed, emitted, and asserted in tests (passes 1–6 closure held through a45573e4).

**TD-VSDD-060 (sibling-site sweep):** PASS for production logic — single canonical
`parse_datetime_to_micros` implementation confirmed. PARTIAL MISS for doc-comment class
in fix-burst-4 (passes-5 corrective closure was incomplete; fixed exhaustively in
fix-burst-5 @a3083468).

**ADR-051 D1–D6 conformance:** PASS — all six decisions confirmed in implementation at
a45573e4.

**Positive-value assertions:** PASS — RGT-017..022 all assert materialized scalar values.
AC-011 TOML-source chain confirmed load-bearing.

**Cite-pin currency (POL-25):** PASS — story v1.4 error-taxonomy pin v2.16 confirmed
correct (closed in pass-5).

---

## Findings and Closure Dispositions

### ADV-P07-LOW-001 (LOW) — infusion_udf.rs: module doc + signature-field doc + coerce_to_typed doc retaining stale stub-phase language

**Finding:** Three doc-comment sites in `infusion_udf.rs` retained stub-phase language
after fix-burst-4:
- The module-level doc for `infusion_udf.rs` referenced `todo!()` stubs and "Red Gate holds"
  patterns that described the pre-implementation state.
- The signature-field doc for the UDF struct contained comments describing expected
  stub behavior ("returns a StringArray") that had been superseded by the actual Timestamp
  and typed-output implementation.
- The `coerce_to_typed` function's doc-comment retained language about stub-phase
  preconditions that no longer applied to the fully-implemented function.

Fix-burst-4 (ADV-P05-OBS-002 closure at a45573e4) had swept the most prominent instances
but did not perform an exhaustive grep across all doc-comment sites, leaving residual
stale-stub-doc language at these three locations.

**Severity:** LOW. No behavioral impact; all substantive code paths are correctly
implemented and tested. The stale language creates misleading documentation that could
confuse future contributors about whether the code is fully implemented.

**Closure:** @a3083468 fix-burst-5 — exhaustive grep-driven sweep of all `infusion_udf.rs`
doc-comment sites rewrote 4 production doc sites and 5 test-body sites. Confirmation grep
for stale-stub-doc language returned empty (only intentional CRIT-3 regression-guard
`todo!()` calls remain, which describe a bug state being guarded, not stub placeholders).

---

### ADV-P07-LOW-002 (LOW) — loader.rs: both validators' BC-5.38.001 self-check blocks retaining stale stub-phase language

**Finding:** Two validator functions in `loader.rs` had their BC-5.38.001 self-check
comment blocks (which are normally removed when a function transitions from stub to
implementation) retained as stale comments. The self-check blocks described the stub
phase's Red Gate enforcement pattern: "BC-5.38.001: this function holds Red Gate until
the implementation is complete." These blocks are correct during the stub phase but
must be removed when the function is fully implemented and passing its tests.

Fix-burst-4 focused on the primary files identified by ADV-P05-OBS-002 (infusion_udf.rs)
and did not perform a cross-file grep for BC-5.38.001 self-check blocks, leaving the
loader.rs instances unaddressed.

**Severity:** LOW. No behavioral impact. Stale self-check blocks can falsely suggest
functions are still in stub phase to reviewers.

**Closure:** @a3083468 fix-burst-5 — both BC-5.38.001 self-check blocks removed from
`loader.rs` validator functions. Verification: grep for `BC-5.38.001` in the affected
files confirms removal.

---

### ADV-P07-OBS-001 (OBS) — test-body todo!() comments in implemented test functions

**Finding:** Several test functions in `infusion_tests.rs` and `enrichment_pivot_002_tests.rs`
retained `todo!()` comments in test bodies that described the pre-implementation expected
behavior. These `todo!()` comments are NOT `todo!()` macro calls (which would cause
compilation failures); they are plain-text comments formatted as `// todo: verify X`
or `// TODO: assert Y value once stub is replaced` that were inserted during the
stub-phase to mark where assertions would go. With the implementation complete and
real assertions in place, these comments are stale noise.

The 27 sites in `enrichment_pivot_002_tests.rs` and 1 site in `infusion_tests.rs`
are the residual set from fix-burst-4's partial sweep.

**Severity:** OBS (observational). No functional impact; assertions are present and
correct. Stale todo-comments reduce test readability.

**Closure:** @a3083468 fix-burst-5 — all 28 stale todo-comment sites removed across
`enrichment_pivot_002_tests.rs` (27 sites) and `infusion_tests.rs` (1 site). Confirmation
grep for stale-pattern strings returned empty across the affected files.

---

## Positive Verifications (all prior closures independently re-derived GREEN at a45573e4)

| Finding | Verification at a45573e4 | Status |
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
| pass-5 LOW-001 (error-taxonomy pin v2.16) | Story §References error-taxonomy cite at v2.16 | HELD |
| pass-5 OBS-001 (extract_at_path numeric-index regression test) | test_extract_at_path_numeric_index_resolves_first_element present and passing | HELD |
| pass-5 OBS-002 (primary stub-phase doc-comments in output_arrow_type/coerce_to_typed) | Primary sites swept at a45573e4 — residual sites closed by fix-burst-5 | PARTIALLY HELD (extended by fix-burst-5) |

---

## Fix-Burst-5 Commit Chain

Fix-burst-5 performed an EXHAUSTIVE grep-driven sweep of all stale-stub-doc language
across the entire diff (not just adversary-cited sites), closing the entire class
root-and-branch:

| File | Sites swept | Change |
|------|------------|--------|
| `prism-spec-engine/src/infusion_udf.rs` | 4 prod doc + 5 test | Module doc, signature-field doc, coerce_to_typed doc, test-body todo-comments rewrote to reflect implemented state |
| `prism-spec-engine/src/loader.rs` | 2 validators | BC-5.38.001 self-check blocks removed from both validator functions |
| `prism-spec-engine/tests/infusion_tests.rs` | 1 | Stale test-body todo-comment removed |
| `prism-spec-engine/tests/enrichment_pivot_002_tests.rs` | 27 | Stale test-body todo-comments removed |

**Total:** 34+ sites swept. Confirmation grep for stale-stub-doc pattern strings returned
empty across all affected files. Only intentional CRIT-3 regression-guard `todo!()` calls
remain (these describe an unresolved bug condition being regression-guarded, not stub
placeholders).

**Code HEAD:** a45573e4→a3083468
**just check result:** GREEN 5213 tests; non-exhaustive 89/89

---

## Process-Gap Lesson

**[process-gap] doc-hygiene fix-bursts must sweep whole diff root-and-branch**

Fix-burst-4 closed ADV-P05-OBS-002 (stale stub-phase doc-comments) by addressing the
adversary-cited sites in `output_arrow_type` and `coerce_to_typed`. Pass-7 found the
same class recurred in `infusion_udf.rs`, `loader.rs`, `infusion_tests.rs`, and
`enrichment_pivot_002_tests.rs` — none of which were cited in the original finding.

The correct discipline (TD-VSDD-060 sibling-site sweep applied to doc-comments, S-7.01
defensive sweep): when closing a doc-hygiene class, grep the ENTIRE diff (not just the
adversary-cited files) for ALL instances of the class pattern, then close all of them in
a single fix-burst. Adversary-cited sites are the MINIMUM; the fix-burst obligation is
the ENTIRE CLASS.

Codification candidate: add to fix-burst pre-close checklist: "For doc-hygiene findings,
performed exhaustive grep of all in-scope files for the class pattern, not just
adversary-cited sites."
