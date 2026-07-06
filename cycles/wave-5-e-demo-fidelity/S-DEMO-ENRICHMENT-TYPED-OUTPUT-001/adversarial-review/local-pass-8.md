---
document_type: adversarial-review
scope: LOCAL
passes: [8]
story: S-DEMO-ENRICHMENT-TYPED-OUTPUT-001
feature_head: a3083468
fix_burst_head: d32bc3af
date: 2026-07-06
clean_strict: false
clean_pr_merge: true
finding_counts: {LOW: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 8 — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

---

## Pass 8 (frozen a3083468)

**Pass result:** CLEAN(PR-merge)=yes, NOT CLEAN(strict) (1 LOW)
**Findings:** 1 LOW
**Code HEAD at review:** a3083468
**Fix-burst HEAD:** d32bc3af
**Post-fix-burst just check:** GREEN 5213 tests; non-exhaustive 89/89
**LOCAL streak after pass-8:** 0/3 (UNCHANGED — pass-8 not strict-clean)
**Next:** LOCAL passes 9/10/11 SEQUENTIAL on frozen d32bc3af

**Finding class:** doc-accuracy class — doc-comment example cited a panic-prone byte-slice
indexing pattern that contradicts the actual char-based implementation, AND the same
inaccuracy appeared in the story's EC-005 error condition wording. Fix-burst-6 chose a
COMPREHENSIVE doc-accuracy audit approach rather than a one-at-a-time fix, because passes
5/7/8 each surfaced a different doc drift. After code converges, one sweep is more efficient
than one adversary pass per doc site.

---

## SAP Probe Results (Pass 8, verified against a3083468)

**SAP-1 (tracing emission catalog completeness):** PASS — no new event_type sites without
BC-2.16.002 catalog row. `infusion.coercion_failed` catalog row confirmed present at v1.95.
All tracing emission sites in crates/ verified covered.

**SAP-2 (DTU↔TOML schema parity):** PASS — cyberint/crowdstrike TOML-declared columns
all have DTU-equivalent fields. No new TOML columns introduced without DTU counterpart.

**TD-VSDD-059 (paper-fix detection):** PASS — `InfusionError::TypeCoercionFailed` is
constructed, emitted, and asserted in tests (passes 1–7 closure held through a3083468).

**TD-VSDD-060 (sibling-site sweep):** PASS — single canonical `parse_datetime_to_micros`
implementation confirmed. Fix-burst-6 swept the ENTIRE diff's doc/comment/example prose
for the doc-accuracy class (no adversary-cited-site-only partial sweep repeat).

**ADR-051 D1–D6 conformance:** PASS — all six decisions confirmed in implementation at
a3083468.

**Positive-value assertions:** PASS — RGT-017..022 all assert materialized scalar values.
AC-011 TOML-source chain confirmed load-bearing.

**Cite-pin currency (POL-25):** FAIL at a3083468 — story body intro + §References cited
ADR-051 v1.3 (stale; post-pass-1 column_type fix; current canonical is v1.4 per ARCH-INDEX
v2.173). CLOSED in fix-burst-6 story v1.5: 4 locations updated v1.3→v1.4.

**Resolver correctness (AC-011 extract_at_path):** PASS — numeric-index regression test
`test_extract_at_path_numeric_index_resolves_first_element` present and passing at a3083468.

---

## Findings and Closure Dispositions

### ADV-P08-LOW-001 (LOW) — coerce_to_typed doc-comment example: byte-slice indexing CWE-248

**Finding:** The doc-comment example for `coerce_to_typed` demonstrated string truncation
using a byte-slice pattern `&value[..50]` (or equivalent byte-index arithmetic). This
pattern is panic-prone (CWE-248) on multibyte UTF-8 characters — slicing at a byte boundary
inside a multi-byte code point causes a runtime panic. The actual production code correctly
uses `value.chars().take(50).collect::<String>()` (char-based truncation, UTF-8-safe).

The same inaccuracy appeared in the story spec's EC-005 error condition description, which
described the truncation implementation as byte-based ("exactly 50 bytes") rather than
char-based ("at most 50 Unicode scalar values" / "at most 50 chars").

This is a doc-accuracy defect in the same class as those surfaced in passes 5 and 7.
The finding is LOW severity because no production logic is affected — all five emission
sites in `infusion_udf.rs` use the correct `chars().take(50)` implementation — but the
doc-comment and story prose both document a different (incorrect and panic-prone) algorithm.

**Severity:** LOW (CWE-248 doc mis-description; no production code affected; correct
implementation already in place).

**Closure:** @d32bc3af fix-burst-6 — coerce_to_typed doc-comment example corrected to
`value.chars().take(50).collect::<String>()`. Story v1.5 EC-005 prose updated to reflect
char-based truncation ("at most 50 chars" / "UTF-8-safe char iteration").

---

## Fix-Burst-6 Comprehensive Doc-Accuracy Audit

Fix-burst-6 chose to perform a COMPREHENSIVE doc-accuracy audit of the entire diff's
docs, comments, and prose rather than a one-at-a-time closure. Rationale: passes 5, 7,
and 8 each surfaced a different doc drift in the same diff; the pattern indicates more
latent inaccuracies beyond the adversary-cited site exist. After code converges, sweeping
all doc/comment/prose in one pass is more efficient than surfacing one per adversary pass.

### Code-side corrections (@d32bc3af — docs/comments/examples only, NO code-logic change)

Five code-documentation sites corrected:

| Site | Change |
|------|--------|
| `coerce_to_typed` doc-comment example | Byte-slice `&value[..50]` → char-based `value.chars().take(50).collect::<String>()` |
| doc example `%declared_type` | Non-existent format variable `%declared_type` → `%self.descriptor.output_type` (the actual field name) |
| doc example missing error argument | `tracing::warn!(...)` example missing `"{}", err` arg → added |
| Cyberint AC-011 test-comment | Wrong regression premise ("numeric coercion inapplicable to string column") → correct premise ("source_path/generator drift: $.iocs[0].value path") |
| `loader::validate_plugin_type_has_source_column` error-format | Placeholder `{name}` → `{field_name}` (matches actual local variable name in error construction) |

**just check result:** GREEN 5213 tests; non-exhaustive 89/89 (docs-only change; test count UNCHANGED)

### Story-side corrections (story-writer v1.5)

| Site | Change |
|------|--------|
| EC-005 truncation description | "byte-slice" / "exactly 50 chars" → "char-based `chars().take(50)`" / "at most 50 chars (UTF-8-safe)" |
| AC-004 point 2 | Added explicit note: `declared_type` = `output_type` spec-vocabulary string (e.g., `"integer"`) — NOT Arrow debug format (`Int64`); truncation implementation cited |
| ADR-051 cite-pin (body intro) | v1.3 → v1.4 (POL-25; ARCH-INDEX v2.173 is canonical; 4 locations updated) |
| ADR-051 cite-pin (§References table) | v1.3 → v1.4 |

All other audit items (22 RGT names, BC versions, JSONPath expressions, E-INFUSE-013
sub-condition wording, ENRICH-1 json-typed-only constraint, resources.rs annotations)
verified CORRECT against code a3083468. No further changes required.

**Code HEAD:** a3083468 → d32bc3af
**just check result:** GREEN 5213 tests; non-exhaustive 89/89

---

## Positive Verifications (all prior closures independently re-derived GREEN at a3083468)

| Finding | Verification at a3083468 | Status |
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
| pass-5 OBS-002 / pass-7 (stale stub-phase doc-comments — exhaustive sweep) | fix-burst-5 exhaustive sweep at a3083468 confirmed empty confirmation grep | HELD |

---

## Process-Gap Lesson

**[process-gap] After code converges: one COMPREHENSIVE doc-accuracy audit, not one finding per adversary pass**

Passes 5, 7, and 8 each surfaced a different doc drift in the same diff:
- Pass 5: error-taxonomy pin stale (story spec POL-25)
- Pass 7: stale stub-phase doc-comment language (infusion_udf.rs, loader.rs, test files)
- Pass 8: byte-slice doc-comment example contradicting char-based implementation (CWE-248)

All three are instances of the same doc-accuracy class: code was implemented correctly
but documentation/comments/prose retained inaccurate descriptions. The correct discipline:

**Once code logic is substantively CLEAN (no code-logic findings), run ONE comprehensive
doc-accuracy audit pass covering the entire diff's docs, comments, examples, and story
prose before advancing the LOCAL cascade. This audit can be combined with an adversary
pass but must sweep ALL doc/comment/prose in the diff, not only adversary-cited sites.**

This extends the pass-7 process-gap (doc-hygiene fix-bursts must sweep whole diff) to
the dispatch level: the audit is a prerequisite sweep, not a reactive-one-at-a-time
fix. Codification candidate: add to the LOCAL adversary dispatch template for stories
where code is substantively clean: "run comprehensive doc-accuracy audit before advancing
3-CLEAN streak."
