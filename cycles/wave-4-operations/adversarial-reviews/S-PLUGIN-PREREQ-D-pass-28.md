---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 28
target_sha: 4e919a74
story_content_sha: 9f342d61
error_taxonomy_content_sha: 8e980a0e
bc_content_sha: 84f58565
base_sha: 95d46be2
verdict: BLOCKED
streak: "0/3 HOLD (pass-28 BLOCKED: 2 MED + 3 LOW + 1 OBS — phantom §-section + wrong trace anchor + Token Budget drift)"
finding_summary: {CRITICAL: 0, HIGH: 0, MEDIUM: 2, LOW: 3, OBS: 1}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7, pass-8, pass-9, pass-10, pass-11, pass-12, pass-13, pass-14, pass-15, pass-16, pass-17, pass-18, pass-19, pass-20, pass-21, pass-22, pass-23, pass-24, pass-25, pass-26, pass-27]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6, fix-burst-7, fix-burst-8, fix-burst-9, fix-burst-10, fix-burst-11, fix-burst-12, fix-burst-13, fix-burst-14, fix-burst-15, fix-burst-16, fix-burst-17, fix-burst-18, fix-burst-19, fix-burst-20, fix-burst-21, fix-burst-22, fix-burst-23, fix-burst-24, fix-burst-25]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → 5"
idempotency_check: false
post_fix_check: true
post_fix_target: "fix-burst-25 (4 fixes; all PASS in regression check)"
producer: "adversary (vsdd-factory; reified by state-manager due to read-only tool profile)"
---

# Adversarial Pass 28 — S-PLUGIN-PREREQ-D

**Verdict: BLOCKED (2 MEDIUM + 3 LOW + 1 OBSERVATION)**

**Context:** This is a post-fix-burst-25 fresh-context pass. Fix-burst-25 closed
4 in-scope findings (F-LP27-MED-001 subsystems SS-16 added; F-LP27-MED-002 PluginError
#[non_exhaustive] MVP-hedge stripped; F-LP27-MED-003 §References BC titles verbatim;
F-LP27-LOW-001 BC-2.17.005 added to inputs). The expected outcome was CLEAN (0/3 → 1/3).
Actual outcome: BLOCKED by 2 MEDIUM + 3 LOW + 1 OBS — new finding classes including
a phantom §-section anchor pattern (codification candidate #14 emerging). Streak holds
at 0/3 per BC-5.39.001.

---

## Codification #13 Cross-Table Sweep Verification

**Target:** Confirm codification #13 (POL-7 cross-table sweep — verify BC title verbatim
at ALL citation sites: body BC table, §References, Architecture Compliance Rules, frontmatter
comments, prose) holds after fix-burst-25.

16 citation sites were swept across story body BC table (8 BCs × title cells), §References
(8 BC title entries), Architecture Compliance Rules (4 cross-table references), and
frontmatter comments (4 YAML anchor labels).

| Site | Citation Location | BC | Title Match | Result |
|------|------------------|----|-------------|--------|
| 1 | Body BC table — BC-2.16.002 Title cell | BC-2.16.002 | "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" verbatim BC H1 | PASS |
| 2 | Body BC table — BC-2.17.001 Title cell | BC-2.17.001 | verbatim BC H1 | PASS |
| 3 | Body BC table — BC-2.17.002 Title cell | BC-2.17.002 | verbatim BC H1 | PASS |
| 4 | Body BC table — BC-2.17.003 Title cell | BC-2.17.003 | verbatim BC H1 | PASS |
| 5 | Body BC table — BC-2.17.004 Title cell | BC-2.17.004 | verbatim BC H1 | PASS |
| 6 | Body BC table — BC-2.17.006 Title cell | BC-2.17.006 | verbatim BC H1 | PASS |
| 7 | Body BC table — BC-2.17.007 Title cell | BC-2.17.007 | verbatim BC H1 (parenthetical annotation preserved) | PASS |
| 8 | Body BC table — BC-2.22.001 Title cell | BC-2.22.001 | verbatim BC H1 | PASS |
| 9 | §References — BC-2.16.002 entry | BC-2.16.002 | verbatim BC H1 | PASS |
| 10 | §References — BC-2.17.001 entry | BC-2.17.001 | verbatim BC H1 | PASS |
| 11 | §References — BC-2.17.002 entry | BC-2.17.002 | verbatim BC H1 | PASS |
| 12 | §References — BC-2.17.003 entry | BC-2.17.003 | verbatim BC H1 | PASS |
| 13 | §References — BC-2.17.004 entry | BC-2.17.004 | verbatim BC H1 | PASS |
| 14 | §References — BC-2.17.006 entry | BC-2.17.006 | verbatim BC H1 | PASS |
| 15 | §References — BC-2.17.007 entry | BC-2.17.007 | verbatim BC H1 (parenthetical annotation preserved) | PASS |
| 16 | §References — BC-2.22.001 entry | BC-2.22.001 | verbatim BC H1 | PASS |

**Codification #13 cross-table sweep: 16/16 PASS — CODIFICATION HELD.**

---

## Regression Check — fix-burst-25 + fix-burst-23 + fix-burst-24

All prior fix-burst closures verified for regression in this pass.

| Prior Fix-Burst | Target Finding | Regression Check |
|-----------------|----------------|-----------------|
| fix-burst-25 F-LP27-MED-001 | subsystems: [SS-22, SS-17, SS-16] | PASS — SS-16 present in story frontmatter |
| fix-burst-25 F-LP27-MED-001 | anchor_subsystem: SS-16 symmetric | PASS — YAML comment + anchor_subsystem updated |
| fix-burst-25 F-LP27-MED-002 | PluginError #[non_exhaustive] prescription unconditional | PASS — MVP-hedge conditional language absent; §non_exhaustive Requirements unconditional |
| fix-burst-25 F-LP27-MED-003 | §References 8/8 BC titles verbatim BC H1 | PASS (codification #13 sweep above confirms 8/8 §References entries) |
| fix-burst-25 F-LP27-LOW-001 | BC-2.17.005-plugin-hot-reload-atomic-swap.md in inputs: | PASS — present in inputs frontmatter |
| fix-burst-24 F-LP26-MED-001 | BC-2.16.002 body-table title verbatim | PASS (codification #13 site 1 confirms) |
| fix-burst-23 F-LP25-HIGH-001 | ADR-023 §C4 absent; BC-2.17.005 §Invariants anchored | PASS — ADR-023 §C4 absent; BC-2.17.005 §Invariants cite intact |
| fix-burst-23 F-LP25-LOW-001 | SS-17 "WASM Plugin Runtime" | PASS — WASM Plugin Runtime present |
| fix-burst-23 F-LP25-LOW-002 | AC-9 fabricated hedge stripped | PASS — canonical BC-2.17.002 §Error Conditions E-PLUGIN-005 intact |

**fix-burst-25 + fix-burst-23 + fix-burst-24 regression check: all PASS.**

---

## POL-22 Phase A — External Anchor Verification (30-Anchor Table)

Per codification candidate #11 discipline: adversary opens and greps cited target
documents; story-body substring match is NOT sufficient.

| Anchor | Story Citation | Target Document | Grep Result |
|--------|---------------|-----------------|-------------|
| BC-2.16.002 §Catalog | AC-3 catalog anchor | BC-2.16.002 §Catalog | PASS |
| BC-2.16.002 v1.12 | AC-3, AC-7, §Catalog Additions intro | BC-2.16.002 frontmatter | PASS |
| BC-2.17.001 §Preconditions | AC-1 | BC-2.17.001 §Preconditions | PASS |
| BC-2.17.001 §Behavior | AC-2 | BC-2.17.001 §Behavior | PASS |
| BC-2.17.002 §Error Conditions E-PLUGIN-005 | AC-9 | BC-2.17.002 §Error Conditions | PASS |
| BC-2.17.002 v1.5 | AC-9 anchor | BC-2.17.002 frontmatter | PASS |
| BC-2.17.003 §Preconditions | AC-4 | BC-2.17.003 §Preconditions | PASS |
| BC-2.17.004 §Invariants | AC-5 | BC-2.17.004 §Invariants | PASS |
| BC-2.17.005 §Invariants (spawn_blocking) | Architecture Compliance Rules row | BC-2.17.005 §Invariants | PASS |
| BC-2.17.005 (inputs) | inputs: frontmatter | inputs: array | PASS (added by fix-burst-25) |
| BC-2.17.006 §Behavior | AC-6 | BC-2.17.006 §Behavior | PASS |
| BC-2.17.007 §Preconditions | AC-8 | BC-2.17.007 §Preconditions | PASS |
| BC-2.22.001 §Behavior | AC-7 allowlist | BC-2.22.001 §Behavior | PASS |
| BC-2.22.001 v1.5 | AC-7 anchor | BC-2.22.001 frontmatter | PASS |
| ADR-022 §C plugin-wiring | AC-10 | ADR-022 §C | PASS |
| ADR-023 §A boot-sequence | AC-11 | ADR-023 §A | PASS |
| ADR-023 §B WASM init | AC-12 | ADR-023 §B | PASS |
| ADR-023 §C1 error propagation | AC-13 | ADR-023 §C | PASS |
| ADR-023 §C2 capability-list | AC-14 | ADR-023 §C | PASS |
| ADR-023 §C3 concurrency | AC-15 | ADR-023 §C | PASS |
| E-PLUGIN-001..012 | error taxonomy rows | error-taxonomy.md | PASS |
| E-PIPELINE-001 | AC-16 rate-limit | error-taxonomy.md PIPELINE namespace | PASS |
| SpecEngineError::TooManyRequests | AC-16 type | prism-spec-engine/src/error.rs | PASS |
| Vec<String> type contract | AC-7, AC-17 | BC-2.22.001 §Behavior contract | PASS |
| HostState::test_default() Vec<String> | AC-17 prescription | Match-Site Inventory all 6 rows | PASS |
| E-PLUGIN-013..016 four-layer chain | AC-5 | BC-2.17.004 + error-taxonomy | PASS |
| AC-16 MAX_REQUESTS_PER_PIPELINE | rate-limit constraint | story body + BC-2.16.002 §Canonical Structured Event Catalog row `pipeline_max_requests_exceeded` | PASS-with-Drift — see F-LP28-MED-002 (AC-16 trace header cites wrong §section) |
| SS-16 subsystem anchor | story frontmatter subsystems: [SS-22, SS-17, SS-16] | ARCH-INDEX SS-16 "Spec Engine" | PASS (fix-burst-25 closed) |
| SS-22 subsystem anchor | story frontmatter line 50 | ARCH-INDEX SS-22 | PASS |
| SS-17 WASM Plugin Runtime | story frontmatter | ARCH-INDEX SS-17 | PASS |

**POL-22 Phase A result: 28/30 PASS — 1 PASS-with-Drift (AC-16 trace anchor) — see F-LP28-MED-002. 1 PASS-with-OBS (STORY-INDEX range drift) — closed in-burst (F-LP28-LOW-002).**

---

## POL-22 Phase B — Internal Cross-Reference Symmetry Chains (5-Chain Table)

| Chain | Chain Elements | Result |
|-------|---------------|--------|
| Vec<String> contract chain | AC-7 declaration → AC-17 body `Vec<String>` → Task 2 reference → `test_default()` `Vec<String>` → all 6 Match-Site rows `Vec<String>` | PASS |
| E-PLUGIN-013/014/015/016 four-layer | AC-5 error reference → Error Taxonomy Additions §E-PLUGIN-013..016 rows → BC-2.17.004 §Invariants → prism-spec-engine error.rs canonical | PASS |
| E-PIPELINE-001 five-layer | AC-16 rate-limit → Error Taxonomy Additions §E-PIPELINE-001 row → error-taxonomy.md PIPELINE namespace → BC-2.16.002 §Canonical Structured Event Catalog `pipeline_max_requests_exceeded` → `SpecEngineError::TooManyRequests` canonical type | PASS-with-Drift — AC-16 trace header cites "BC-2.16.002 preconditions" but the cap appears in §Catalog row, not §Preconditions; see F-LP28-MED-002 |
| Manifest four-code symmetry | AC-3 manifest fields → §Structured Event Catalog Additions 9 rows → BC-2.16.002 §Catalog entries → wasm-manifest.toml schema | PASS |
| BC-title-verbatim sweep (codification #12 body + codification #13 cross-table) | Each of 8 BC rows in story body BC table + §References + Architecture Compliance Rules: Title cell vs BC H1 | 16/16 PASS — all citation sites verbatim (see codification #13 table above) |

**POL-22 Phase B result: 4/5 PASS — 1 PASS-with-Drift (E-PIPELINE-001 chain: AC-16 trace header wrong §section)**

---

## Phase C — Carry-Forward Sample (13-Sample Table)

| Prior Finding | Fix Applied | Regression Check |
|---------------|-------------|-----------------|
| F-LP1-HIGH-001 (missing AC-7 allowlist) | Added AC-7 with Vec<String> contract | PASS — AC-7 present, Vec<String> intact |
| F-LP4-HIGH-001 (manifest schema gaps) | Added wasm-manifest.toml fields | PASS — fields present in §Structured Event Catalog |
| F-LP7-HIGH-001 (E-PLUGIN-001 error code missing) | Added E-PLUGIN-001..012 taxonomy rows | PASS — all 12 rows present |
| F-LP8-HIGH-001 (spawn_blocking scope gap) | Added Architecture Compliance Rules table | PASS — table present; spawn_blocking in BC-2.17.005 §Invariants |
| F-LP9-HIGH-001 (capability gate doc gap) | Added AC-14 capability-list section | PASS — AC-14 present, ADR-023 §C2 anchor intact |
| F-LP11-HIGH-001 (concurrency permit leak) | Added AC-15 concurrency section | PASS — AC-15 present, ADR-023 §C3 anchor intact |
| F-LP15-HIGH-001 / F-LP16-HIGH-001 (PrismError→SpecEngineError) | Corrected type to SpecEngineError | PASS — no PrismError::PluginRuntimeInit in active body |
| F-LP18-HIGH-001 (version pin drift BC-2.16.002 v1.10→v1.12) | Updated 3 version pins | PASS — no v1.10 or v1.11 stale pins in active body |
| F-LP23-HIGH-001 (Option<Vec<String>> type regression 8 sites) | Replaced Option<Vec> → Vec at all 8 sites | PASS — no Option<Vec<String>> in active body |
| F-LP25-HIGH-001 (spawn_blocking re-anchor) | ADR-023 §C4 removed; BC-2.17.005 §Invariants anchored | PASS — ADR-023 §C4 absent from story body |
| F-LP26-MED-001 (BC-2.16.002 body-table title paraphrase) | Verbatim BC H1 applied; 8/8 sweep CLEAN | PASS — verbatim title present (codification #13 site 1) |
| F-LP27-MED-001 (subsystems missing SS-16) | subsystems: [SS-22, SS-17, SS-16] | PASS — SS-16 present and anchor_subsystem symmetric |
| F-LP27-MED-003 (§References BC title paraphrases) | All 8 §References BC titles verbatim | PASS — codification #13 sites 9-16 all PASS |

**Phase C carry-forward sample: 13/13 PASS — no regressions.**

---

## Phase D — New Findings

### F-LP28-MED-001 [process-gap] — Phantom §-Section Reference in Story Body + error-taxonomy.md Sibling (POL-4)

**Severity:** MEDIUM × 2 (one finding pattern; two propagation sites)
**Confidence:** HIGH
**Category:** POL-4 violation (semantic_anchoring_integrity); fabricated §-section anchor;
[process-gap] — codification candidate #14 (phantom-section-anchor sweep)

**Locations:**
- `.factory/stories/S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md` line 918
- `.factory/specs/prd-supplements/error-taxonomy.md` line 464 (sibling-site propagation)

**Finding:**

Both sites contain the citation:
```
Traces to BC-2.16.002 §S-PLUGIN-PREREQ-D AC-16.
```

The `§X` notation conventionally references a section heading inside the cited document.
Searching BC-2.16.002 for `§S-PLUGIN-PREREQ-D` or any heading matching `S-PLUGIN-PREREQ-D`
returns zero heading matches. The only `S-PLUGIN-PREREQ-D` occurrences in BC-2.16.002 are
in changelog rows (lines 165 and 166), NOT in any section heading.

BC-2.16.002 does NOT have a §S-PLUGIN-PREREQ-D section. The citation is a fabricated
section anchor — the §X notation implies a navigable section that does not exist.

The semantically correct anchor is BC-2.16.002 §Canonical Structured Event Catalog row
`pipeline_max_requests_exceeded`, which is where the 10K cap, the event name, and the
MAX_REQUESTS_PER_PIPELINE enforcement documentation actually live (BC-2.16.002 line 102
area). AC-16 of S-PLUGIN-PREREQ-D IS the story that INTRODUCES this enforcement;
BC-2.16.002 documents the emission consequence in §Catalog.

**Routing:**
- Story body (line 918): story-writer fix-burst-26
- error-taxonomy.md (line 464): product-owner fix-burst-26

**Suggested fix:**

Replace both sites:
```
"Traces to BC-2.16.002 §S-PLUGIN-PREREQ-D AC-16"
```
with:
```
"Traces to BC-2.16.002 §Canonical Structured Event Catalog row pipeline_max_requests_exceeded (anchored by AC-16 of S-PLUGIN-PREREQ-D)"
```

**Why this is a codification candidate (#14):**

The `§X` notation must resolve to an actual section heading in the cited document. Any
`§X` citation where X cannot be found as a heading level (H1/H2/H3/H4) in the cited
document is a phantom-section-anchor. This is a new sub-class of POL-4 violations.
Future POL-22 Phase B extension or POL-7 amendment: when verifying BC citations, also
verify that any accompanying §Section notation resolves to an actual heading in the
cited document (grep for `# X`, `## X`, `### X`, etc.).

---

### F-LP28-MED-002 — Wrong Trace Anchor "BC-2.16.002 preconditions" in AC-16 (POL-4)

**Severity:** MEDIUM
**Confidence:** HIGH
**Category:** POL-4 violation (semantic_anchoring_integrity); wrong §section cited

**Location:** `.factory/stories/S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md` line 466
(AC-16 trace header)

**Finding:**

AC-16 trace header cites:
```
Traces to BC-2.16.002 preconditions; introduces MAX_REQUESTS_PER_PIPELINE enforcement
```

BC-2.16.002 §Preconditions (lines 45-50 of BC file) contains:
- Spec-driven table registration (BC-2.16.001 anchor)
- Query dispatch (CAP-015)
- AuthProvider eager-token

BC-2.16.002 §Preconditions does NOT contain MAX_REQUESTS_PER_PIPELINE, the 10K cap, or
any HTTP-request count limit. The 10K cap is documented at BC-2.16.002 line 102 area in
§Canonical Structured Event Catalog — specifically in the `pipeline_max_requests_exceeded`
catalog row which defines: the event emission conditions, the cap limit, and the
`SpecEngineError::TooManyRequests` consequence.

The cap is being INTRODUCED by AC-16; BC-2.16.002 §Catalog documents the emission
consequence of enforcement. Citing §Preconditions points the reader to the wrong section
— there is nothing about MAX_REQUESTS_PER_PIPELINE at that location.

**Routing:** story-writer fix-burst-26

**Suggested fix:**

Replace in AC-16 trace header:
```
"BC-2.16.002 preconditions"
```
with:
```
"BC-2.16.002 §Canonical Structured Event Catalog row pipeline_max_requests_exceeded"
```

---

### F-LP28-LOW-001 — Token Budget BC Row Count Drift 8 → 9

**Severity:** LOW
**Confidence:** HIGH
**Category:** Consistency drift; sibling-site-sweep miss from fix-burst-25 inputs append

**Location:** Story line 678 (Token Budget table)

**Finding:**

Token Budget table reads:
```
| BC files (8 BCs × ~1,500) | ~12,000 |
```

The story `inputs:` array (lines 96-113) now lists 9 BC files — BC-2.17.005 was added
by fix-burst-25 (F-LP27-LOW-001). The Token Budget table was not updated when inputs:
was amended. Current correct values:
- BC files: 9 BCs × ~1,500 = ~13,500
- Total context budget: ~40,900 → ~42,400 (delta +1,500)
- Percentage of 256K context: ~42,400 / 256,000 = ~16.6% (was 16.0%)

**Routing:** story-writer fix-burst-26

**Suggested fix:** Update Token Budget table row 7 (BC files): `8 BCs × ~1,500 → ~12,000` to `9 BCs × ~1,500 → ~13,500`. Update total: `~40,900 → 16.0%` to `~42,400 → 16.6%`.

---

### F-LP28-LOW-002 — STORY-INDEX Line 394 Narrative Range "BC-2.17.001..005" (should be "..004") — CLOSED IN-BURST

**Severity:** LOW
**Confidence:** HIGH
**Category:** STORY-INDEX narrative drift; mismatched range vs explicit BC-list parenthetical
**Status:** CLOSED in this burst (state-manager, D-519 precedent: D-513 STORY-INDEX date drift)

**Location:** `.factory/stories/STORY-INDEX.md` line 394

**Finding:**

STORY-INDEX line 394 PREREQ-D narrative cites:
```
BCs BC-2.16.002 + BC-2.17.001..005 + BC-2.17.006 + BC-2.17.007 + BC-2.22.001
```

The explicit BC-list parenthetical in the same row enumerates 8 BCs:
```
(BC-2.16.002, BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004, BC-2.17.006, BC-2.17.007, BC-2.22.001)
```

BC-2.17.005 is ABSENT from the explicit BC-list parenthetical — correctly, because the
story `behavioral_contracts:` frontmatter array contains only 8 BCs (excluding BC-2.17.005,
which appears only in `inputs:` as a reference dependency, not as a behavioral contract).
The range shorthand `BC-2.17.001..005` is wrong: it implies 5 BCs (BC-2.17.001 through
BC-2.17.005 inclusive), but only 4 are in the behavioral_contracts list (001 through 004).

The range should be `BC-2.17.001..004`.

**Routing:** state-manager in this burst (D-513 precedent: STORY-INDEX date drift fixed
in-burst by state-manager; narrative consistency gap parallel to that precedent).

**Fix applied in this burst:** STORY-INDEX line 394 changed `BC-2.17.001..005` to
`BC-2.17.001..004`. STORY-INDEX version bumped v2.94 → v2.95.

---

### F-LP28-LOW-003 — `inputs:` Missing ADR-022 Despite Extensive Citation

**Severity:** LOW
**Confidence:** HIGH
**Category:** Frontmatter completeness gap; sibling-site pattern to F-LP27-LOW-001 inputs miss

**Location:** Story frontmatter `inputs:` array (lines 96-113)

**Finding:**

ADR-022 is cited at story body lines approximately 70, 79, 387, 393, 649, 650, 859 and
approximately 10 additional sites (ADR-022 §C plugin-wiring, §C1 error propagation, §C2
capability-list, §C3 concurrency, §D boot-sequence cross-ref, Mermaid participant labels,
Architecture Compliance Rules, etc.). ADR-022 is the second-most cited ADR in the story.

However, ADR-023 IS listed in `inputs:` (line 112: `adr-023-wasm-plugin-isolation.md`),
while ADR-022 is absent from `inputs:` despite more citation sites than ADR-023.

ADR-022 should be listed in `inputs:` as:
```yaml
  - .factory/specs/architecture/adr/ADR-022-dependency-injection-constructor-wiring.md
```

This is a sibling-site pattern to F-LP27-LOW-001 (BC-2.17.005 in inputs miss). Fix-burst-25
closed F-LP27-LOW-001 for BC-2.17.005; this finding is the ADR-022 parallel.

**Routing:** story-writer fix-burst-26

---

### F-LP28-OBS-001 [process-gap] — E-INT-001 Absent from error-taxonomy.md

**Severity:** OBSERVATION (out-of-perimeter)
**Confidence:** HIGH
**Category:** Cross-document governance gap; pre-existing error taxonomy omission

**Location:** Story body line 393 (cites E-INT-001 referencing `error.rs:881-883`)

**Finding:**

Story line 393 cites E-INT-001 as a valid error code with an implementation reference
(`error.rs:881-883`). The grep confirms E-INT-001 IS present in `crates/prism-spec-engine/src/error.rs`
at lines 881-883 (real code). However, E-INT-001 does NOT appear in
`.factory/specs/prd-supplements/error-taxonomy.md`. The error taxonomy has no E-INT-NNN
namespace documented.

This is a pre-existing gap — E-INT-001 was not introduced by this story (the story cites
it as an existing error code). The error-taxonomy.md E-INT namespace is entirely absent.

**Why it is out-of-perimeter:**

Adding a new namespace (E-INT) to error-taxonomy.md requires product-owner adjudication
and is a cross-doc governance task broader than the story scope. The story correctly cites
E-INT-001 per the existing codebase; the gap is in the taxonomy artifact, not in the story.

**Routing:** product-owner phase-5 deferred. Route for taxonomy amendment
(`deferred-findings-phase-5.md`). (6th Phase-5 deferred finding.)

---

## Codification Candidate #14

**Pattern:** Phantom-section-anchor sweep — `§X` notation must resolve to an actual section
heading (H1/H2/H3/H4 or equivalent) in the cited document.

**Evidence:** F-LP28-MED-001 — both the story body (line 918) and the error-taxonomy.md
sibling (line 464) cite `BC-2.16.002 §S-PLUGIN-PREREQ-D AC-16` where `§S-PLUGIN-PREREQ-D`
is not a section heading in BC-2.16.002 — only changelog rows match.

**Proposed codification:** Extend POL-22 Phase B or add POL-7 amendment: when verifying
BC citations with accompanying `§Section` notation, adversary must grep the cited document
for the section heading text (not just the BC ID). If no heading match is found, the citation
is a phantom-section-anchor (POL-4 violation).

**Tag:** For cycle-close session-reviewer adjudication (14th candidate).

---

## Summary

Pass-28 returned 5 findings + 1 OBS from fresh-context audit at story v1.25
(SHA 9f342d61) + BC-2.16.002 v1.12 (SHA 84f58565) + error-taxonomy v1.20 (SHA 8e980a0e).

| Finding | Severity | Routing | Status |
|---------|----------|---------|--------|
| F-LP28-MED-001 | MEDIUM (×2 sites) | story-writer (story:918) + product-owner (error-taxonomy:464) | fix-burst-26 |
| F-LP28-MED-002 | MEDIUM | story-writer (story:466) | fix-burst-26 |
| F-LP28-LOW-001 | LOW | story-writer (story:678) | fix-burst-26 |
| F-LP28-LOW-002 | LOW | state-manager | CLOSED in-burst (v2.94→v2.95) |
| F-LP28-LOW-003 | LOW | story-writer (story frontmatter inputs:) | fix-burst-26 |
| F-LP28-OBS-001 | OBS [process-gap] | product-owner | phase-5 deferred |

Streak: 0/3 HOLD (BC-5.39.001; BLOCKED resets streak).

Codification #13 cross-table sweep HELD: 16/16 PASS.
Codification #14 (phantom-section-anchor sweep) raised.
Regression check: all prior fix-burst closures CLEAN (0 regressions).

fix-burst-26 routes:
- story-writer: F-LP28-MED-001 story:918 + F-LP28-MED-002 AC-16 trace header + F-LP28-LOW-001 Token Budget + F-LP28-LOW-003 inputs ADR-022
- product-owner: F-LP28-MED-001 error-taxonomy:464 sibling
- state-manager: F-LP28-OBS-001 → deferred-findings-phase-5.md (done in D-519 burst)
