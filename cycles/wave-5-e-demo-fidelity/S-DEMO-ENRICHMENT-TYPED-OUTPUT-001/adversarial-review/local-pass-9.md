---
document_type: adversarial-review
scope: LOCAL
passes: [9]
story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
feature_head: d32bc3af
fix_burst_head: 11784b57
date: 2026-07-06
clean_strict: false
clean_pr_merge: true
finding_counts: {OBS: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 9 — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

---

## Pass 9 (frozen d32bc3af)

**Pass result:** CLEAN(PR-merge)=yes, NOT CLEAN(strict) (1 OBS)
**Findings:** 1 OBS
**Code HEAD at review:** d32bc3af
**Fix-burst HEAD:** 11784b57
**Post-fix-burst just check:** GREEN 5213 tests; non-exhaustive 89/89
**LOCAL streak after pass-9:** 0/3 (UNCHANGED — pass-9 not strict-clean)
**Next:** LOCAL passes 10/11/12 SEQUENTIAL on frozen 11784b57

**Finding class:** doc-hygiene class — stale volatile line-pin "(lines 198-206)" referencing a
specific code-file line range, PLUS false present-tense language "return_type() is hardcoded to
Utf8" in a test doc-comment. Both are in the same doc-comment block for a Red Gate test that
exercises the return_type() method. The code itself is correct; the doc-comment describes the
pre-fix state as if it were still the present state. Fix-burst-7 performed an EXHAUSTIVE
residual doc-class sweep rather than a one-site fix, because passes 5/7/8/9 each surfaced a
different doc-comment nit — indicating latent doc-class residuals remained after prior sweeps.

---

## SAP Probe Results (Pass 9, verified against d32bc3af)

**SAP-1 (tracing emission catalog completeness):** PASS — no new event_type sites without
BC-2.16.002 catalog row. `infusion.coercion_failed` catalog row confirmed present at v1.95.
All tracing emission sites in crates/ verified covered. fix-burst-7 is docs/comments-only;
no new emission sites introduced.

**SAP-2 (DTU↔TOML schema parity):** PASS — cyberint/crowdstrike TOML-declared columns
all have DTU-equivalent fields. fix-burst-7 changes no TOML specs.

**TD-VSDD-059 (paper-fix detection):** PASS — `InfusionError::TypeCoercionFailed` is
constructed, emitted, and asserted in tests (passes 1-8 closure held through d32bc3af).
fix-burst-7 is docs-only; no closures affected.

**TD-VSDD-060 (sibling-site sweep):** PASS — fix-burst-7 performed an exhaustive sweep
across ALL sites in the diff (infusion_udf.rs 10 sites + enrichment_pivot_002_tests.rs 15
sites + pipeline.rs 41 sites = 66 total). Not a one-adversary-cited-site-only fix.

**ADR-051 D1-D6 conformance:** PASS — all six decisions confirmed in implementation at
d32bc3af.

**Positive-value assertions:** PASS — RGT-017..022 all assert materialized scalar values.
AC-011 TOML-source chain confirmed load-bearing.

**Cite-pin currency (POL-25):** PASS — all spec citations at canonical versions; ADR-051
v1.4; BC-2.16.002 v1.95; error-taxonomy v2.16.

---

## Findings and Closure Dispositions

### ADV-P09-OBS-001 (OBS) — test doc-comment: stale volatile line-pin + false present-tense

**Finding:** The doc-comment for a Red Gate test (`test_return_type_matches_output_type_for_all_declared_types`
or equivalent) contained two doc-hygiene defects:

1. **Stale volatile line-pin:** The comment referenced a specific code location as
   "(lines 198-206)" — a volatile line-number citation that becomes incorrect on any
   subsequent code change. This is a TD-VSDD-091 violation (anti-volatile-pin discipline).

2. **False present-tense language:** The comment stated "return_type() is hardcoded to Utf8"
   — describing the pre-fix state (before ADR-051 typed output was implemented) as if it
   were the current behavior. The actual current behavior is the opposite: return_type()
   returns the declared output type from the spec. This is a doc-accuracy defect of the
   same class surfaced in passes 5, 7, and 8.

This is an OBS (observation) severity because no production logic is affected. The test
itself is correct and load-bearing. The doc-comment is wrong about what the code does.

**Severity:** OBS (doc-hygiene / TD-VSDD-091; no production logic affected).

**Closure:** @11784b57 fix-burst-7 — exhaustive residual doc-class sweep. 66 sites across
the full diff reviewed and corrected:
- `infusion_udf.rs` (10 sites): volatile line-pins removed; false present-tense
  "hardcoded" / "Red Gate holds" language reworded to past-tense pre-fix framing.
- `enrichment_pivot_002_tests.rs` (15 sites): same doc-class sweep.
- `pipeline.rs` (41 sites): same doc-class sweep.
- Residual grep confirmed clean after fix-burst-7.
- just check GREEN 5213; non-exhaustive 89/89 (docs-only change; test count UNCHANGED).

---

## Fix-Burst-7 Exhaustive Residual Doc-Class Sweep

Fix-burst-7 chose to perform an EXHAUSTIVE residual doc-class sweep across the entire diff
rather than a one-site fix. Rationale: passes 5, 7, 8, and 9 each surfaced a different
doc-comment nit in the same diff; the pattern indicates the doc-class has not been
fully eradicated by prior fix-burst sweeps.

### Doc-class instances swept (@11784b57 — docs/comments only, NO code-logic change)

| File | Sites | Change type |
|------|-------|-------------|
| `infusion_udf.rs` | 10 | Volatile line-pins removed; false present-tense "hardcoded" / "Red Gate holds" reworded to past-tense |
| `enrichment_pivot_002_tests.rs` | 15 | Same doc-class sweep |
| `pipeline.rs` | 41 | Same doc-class sweep |
| **Total** | **66** | |

**Confirmation grep:** Grep for residual doc-class patterns (volatile line-pins, "hardcoded to",
"Red Gate holds", "(lines NNN-NNN)") returned empty after fix-burst-7. Doc-class considered
exhausted within this diff.

**just check result:** GREEN 5213 tests; non-exhaustive 89/89 (docs-only change; test count UNCHANGED)

---

## Positive Verifications (all prior closures independently re-derived GREEN at d32bc3af)

| Finding | Verification at d32bc3af | Status |
|---------|--------------------------|--------|
| pass-1 HIGH-001 (SAP-2 cyberint surface) | Real e2e adapter test present and passing | HELD |
| pass-1 MED-001 (TypeCoercionFailed emitted) | TypeCoercionFailed constructed+emitted+asserted (TD-VSDD-059) | HELD |
| pass-1 MED-002 (declared_type=output_type.as_str()) | All 5 sites confirmed correct | HELD |
| pass-1 LOW-001 (sub-cond7 message) | Sub-condition 7 message includes "output_type" | HELD |
| pass-1 LOW-002 (RGT-012/013/014 structural TOML-parse) | Structural parse approach confirmed | HELD |
| pass-1 LOW-003 (AC-002 grep tightened) | Narrow grep pattern confirmed in story v1.5 | HELD |
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
| pass-3 OBS-003 (resources.rs annotation) | Generic placeholder annotation present in story v1.5 | HELD |
| pass-5 LOW-001 (error-taxonomy pin v2.16) | Story §References error-taxonomy cite at v2.16 | HELD |
| pass-5 OBS-001 (extract_at_path numeric-index regression test) | test_extract_at_path_numeric_index_resolves_first_element present and passing | HELD |
| pass-5 OBS-002 / pass-7 / pass-8 (doc-hygiene sweep) | fix-burst-7 exhaustive 66-site sweep at 11784b57 confirmed empty confirmation grep | HELD |
| pass-8 ADV-P08-LOW-001 (byte-slice doc example) | coerce_to_typed doc-comment shows chars().take(50) | HELD |

---

## Process-Gap Lesson

**[process-gap] DOC-HYGIENE ASYMPTOTE: run ONE comprehensive doc audit before starting the 3-CLEAN streak**

Passes 5, 7, 8, and 9 each surfaced a different trivial doc-comment nit in the same diff:
- Pass 5: error-taxonomy pin stale (POL-25)
- Pass 7: stale stub-phase doc-comment language (infusion_udf.rs, loader.rs, test files)
- Pass 8: byte-slice doc-comment example contradicting char-based implementation (CWE-248)
- Pass 9: volatile line-pin + false present-tense in test doc-comment

All four are instances of the same doc-hygiene class: code was implemented correctly but
documentation/comments/prose retained inaccurate or stale descriptions. The fix-burst-7
exhaustive sweep (66 sites) is the final attempt to close this class within the diff.

**Codification candidate:** Once code logic is substantively CLEAN (no code-logic findings),
run ONE comprehensive doc+comment+example audit pass covering ALL sub-classes before
advancing the LOCAL 3-CLEAN streak:
- Stub-phase language ("TODO", "Red Gate holds", placeholder comments)
- Code examples (verify examples match actual implementation)
- Line-number pins (volatile citations per TD-VSDD-091)
- Present-tense claims about pre-fix behavior
- Cite-pin currency (version pins per POL-25)

This extends the pass-7 and pass-8 process-gaps to the dispatch level: the audit is a
prerequisite sweep, not a reactive-one-at-a-time fix. Estimate: one focused pass over all
doc/comment/prose in the diff, without needing an adversary for the non-logic items.
