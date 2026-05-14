---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 27
target_sha: 9a18c2bd
story_content_sha: 45ae2c2f
error_taxonomy_content_sha: 8e980a0e
bc_content_sha: 84f58565
base_sha: 95d46be2
verdict: BLOCKED
streak: "0/3 HOLD (pass-27 BLOCKED: 3 MED + 1 LOW + 1 OBS — codification #12 swept body BC table but §References paraphrase pattern survived)"
finding_summary: {CRITICAL: 0, HIGH: 0, MEDIUM: 3, LOW: 1, OBS: 1}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7, pass-8, pass-9, pass-10, pass-11, pass-12, pass-13, pass-14, pass-15, pass-16, pass-17, pass-18, pass-19, pass-20, pass-21, pass-22, pass-23, pass-24, pass-25, pass-26]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6, fix-burst-7, fix-burst-8, fix-burst-9, fix-burst-10, fix-burst-11, fix-burst-12, fix-burst-13, fix-burst-14, fix-burst-15, fix-burst-16, fix-burst-17, fix-burst-18, fix-burst-19, fix-burst-20, fix-burst-21, fix-burst-22, fix-burst-23, fix-burst-24]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4"
idempotency_check: false
post_fix_check: true
post_fix_target: "fix-burst-24 (F-LP26-MED-001 BC-2.16.002 body-table title canonicalized verbatim BC H1)"
producer: "adversary (vsdd-factory; reified by state-manager due to read-only tool profile)"
---

# Adversarial Pass 27 — S-PLUGIN-PREREQ-D

**Verdict: BLOCKED (3 MEDIUM + 1 LOW + 1 OBSERVATION)**

**Context:** This is a post-fix-burst-24 fresh-context pass. Fix-burst-24 closed
F-LP26-MED-001 (BC-2.16.002 body-table Title cell canonicalized verbatim BC H1;
codification candidate #12). The expected outcome was CLEAN (0/3 → 1/3).
Actual outcome: BLOCKED by 3 MEDIUM + 1 LOW — finding classes that 26 prior
passes missed. Codification candidate #13 surfaces (POL-7 cross-table sweep:
§References section paraphrase pattern, sibling to codification #12 which swept
body BC table only). Streak holds at 0/3 per BC-5.39.001.

---

## Special Verification A — F-LP25-HIGH-001: spawn_blocking Re-Anchor

**Target:** BC-2.17.005 §Invariants must contain `spawn_blocking` rule; story body
must NOT cite `ADR-023 §C4` as the source.

**Check A-1:** Grep BC-2.17.005 for `spawn_blocking`
- Result: `spawn_blocking` IS present at BC-2.17.005 §Invariants (lines 51 and 73).
- Verdict: **PASS**

**Check A-2:** Grep active story body for `ADR-023 §C4`
- Result: `ADR-023 §C4` is ABSENT from active story body (Architecture Compliance Rules
  row and AC-9 and all other sites).
- Verdict: **PASS**

**Overall Special Verification A: PASS — F-LP25-HIGH-001 fix held.**

---

## Special Verification B — F-LP25-LOW-001: SS-17 Short-Name Normalization

**Target:** SS-17 YAML comment must read "WASM Plugin Runtime" (not bare "Plugin Runtime").

**Check B-1:** Grep active story body for `(Plugin Runtime,`
- Result: `(Plugin Runtime,` is ABSENT from active story body.
- Verdict: **PASS**

**Check B-2:** Grep active story body for `(WASM Plugin Runtime,`
- Result: `(WASM Plugin Runtime,` IS present at SS-17 row.
- Verdict: **PASS**

**Overall Special Verification B: PASS — F-LP25-LOW-001 fix held.**

---

## Special Verification C — F-LP25-LOW-002: AC-9 Fabricated Hedge Strip

**Target:** AC-9 trace header must NOT contain fabricated prose referencing
"ADR-023 §C4 plugin HTTP defaults".

**Check C-1:** Grep active story body AC-9 region for `ADR-023 §C4 plugin HTTP defaults`
- Result: fabricated prose IS absent from AC-9 trace header.
- Verdict: **PASS**

**Check C-2:** Canonical BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005 reference intact
- Result: canonical reference is present and unmodified.
- Verdict: **PASS**

**Overall Special Verification C: PASS — F-LP25-LOW-002 fix held.**

---

## Special Verification D — F-LP26-MED-001 (fix-burst-24): BC-2.16.002 Body-Table Title

**Target:** Story body BC table row for BC-2.16.002 must read verbatim BC H1 canonical
title "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation".

**Check D-1:** Grep story body BC table for old paraphrase "Multi-Step Fetch Pipeline — Structured Event Catalog"
- Result: old paraphrase IS absent from story body.
- Verdict: **PASS**

**Check D-2:** Grep story body BC table for verbatim canonical title
- Result: "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" IS present at BC-2.16.002 row.
- Verdict: **PASS**

**Overall Special Verification D: PASS — F-LP26-MED-001 fix-burst-24 closure HELD.**

---

## POL-22 Phase A — External Anchor Verification (30-Anchor Table)

All 30 external anchors verified via fresh-context grep of cited target documents.
Per codification candidate #11 discipline, adversary opens and greps cited target
documents — story-body substring match is NOT sufficient.

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
| AC-16 MAX_REQUESTS_PER_PIPELINE | rate-limit constraint | BC-2.16.002 §Preconditions | PASS |
| SS-16 subsystem anchor | BC-INDEX BC-2.16.002 row | ARCH-INDEX SS-16 "Spec Engine" | CONDITIONAL — see F-LP27-MED-001 |
| SS-22 subsystem anchor | story frontmatter line 50 | ARCH-INDEX SS-22 | PASS |
| SS-17 WASM Plugin Runtime | story frontmatter line 50 | ARCH-INDEX SS-17 | PASS |
| BC-2.17.005 body cite | Architecture Compliance Rules line ~980 | BC-2.17.005 §Invariants | PASS (cite confirmed) — but see F-LP27-LOW-001 (inputs gap) |

**POL-22 Phase A result: 28/30 PASS — 1 CONDITIONAL (SS-16 anchor gap) + 1 Note (BC-2.17.005 inputs gap)**

---

## POL-22 Phase B — Internal Cross-Reference Symmetry Chains (5-Chain Table)

**Note:** Phase B now includes the codification #12 BC-title-verbatim chain per
the mandatory extension agreed at pass-26.

| Chain | Chain Elements | Result |
|-------|---------------|--------|
| Vec<String> contract chain | AC-7 declaration → AC-17 body `Vec<String>` → Task 2 reference → `test_default()` `Vec<String>` → all 6 Match-Site rows `Vec<String>` | PASS |
| E-PLUGIN-013/014/015/016 four-layer | AC-5 error reference → Error Taxonomy Additions §E-PLUGIN-013..016 rows → BC-2.17.004 §Invariants → prism-spec-engine error.rs canonical | PASS |
| E-PIPELINE-001 five-layer | AC-16 rate-limit → Error Taxonomy Additions §E-PIPELINE-001 row → error-taxonomy.md PIPELINE namespace → BC-2.16.002 §preconditions MAX_REQUESTS cap → `SpecEngineError::TooManyRequests` canonical type | PASS |
| Manifest four-code symmetry | AC-3 manifest fields → §Structured Event Catalog Additions 9 rows → BC-2.16.002 §Catalog entries → wasm-manifest.toml schema | PASS |
| BC-title-verbatim sweep (codification #12 — body BC table) | Each of 8 BC rows in story body BC table: Title cell vs BC H1 | 8/8 PASS — fix-burst-24 closure held; BC-2.16.002 verbatim; all others carry-forward CLEAN |

**POL-22 Phase B result: 5/5 PASS**

---

## Phase C — Carry-Forward Sample (13-Sample Table)

13 representative closures from prior passes verified for regression.

| Prior Finding | Fix Applied | Regression Check |
|---------------|-------------|-----------------|
| F-LP1-HIGH-001 (missing AC-7 allowlist) | Added AC-7 with Vec<String> contract | PASS — AC-7 present, Vec<String> intact |
| F-LP4-HIGH-001 (manifest schema gaps) | Added wasm-manifest.toml fields | PASS — fields present in §Structured Event Catalog |
| F-LP7-HIGH-001 (E-PLUGIN-001 error code missing) | Added E-PLUGIN-001..012 taxonomy rows | PASS — all 12 rows present |
| F-LP8-HIGH-001 (spawn_blocking scope gap) | Added Architecture Compliance Rules table | PASS — table present; spawn_blocking in BC-2.17.005 |
| F-LP9-HIGH-001 (capability gate doc gap) | Added AC-14 capability-list section | PASS — AC-14 present, ADR-023 §C2 anchor intact |
| F-LP11-HIGH-001 (concurrency permit leak) | Added AC-15 concurrency section | PASS — AC-15 present, ADR-023 §C3 anchor intact |
| F-LP15-HIGH-001 / F-LP16-HIGH-001 (PrismError→SpecEngineError) | Corrected type to SpecEngineError | PASS — no PrismError::PluginRuntimeInit in active body |
| F-LP18-HIGH-001 (version pin drift BC-2.16.002 v1.10→v1.12) | Updated 3 version pins | PASS — no v1.10 or v1.11 stale pins in active body |
| F-LP23-HIGH-001 (Option<Vec<String>> type regression 8 sites) | Replaced Option<Vec> → Vec at all 8 sites | PASS — no Option<Vec<String>> in active body |
| F-LP25-HIGH-001 (spawn_blocking re-anchor) | ADR-023 §C4 removed; BC-2.17.005 §Invariants anchored | PASS — verified in Special Verification A |
| F-LP25-LOW-001 (SS-17 short-name) | "Plugin Runtime" → "WASM Plugin Runtime" | PASS — verified in Special Verification B |
| F-LP25-LOW-002 (AC-9 fabricated hedge) | AC-9 stripped; BC-2.17.002 v1.5 §Error Conditions retained | PASS — verified in Special Verification C |
| F-LP26-MED-001 (BC-2.16.002 body-table title paraphrase) | Verbatim BC H1 applied; 8/8 sweep CLEAN | PASS — verified in Special Verification D |

**Phase C carry-forward sample: 13/13 PASS — no regressions.**

---

## Phase D — New Findings

### F-LP27-MED-001 — `subsystems:` array omits SS-16 despite BC-2.16.002 anchoring (POL-4)

**Severity:** MEDIUM
**Confidence:** HIGH
**Category:** POL-4 violation (semantic_anchoring_integrity); frontmatter gap

**Location:** `.factory/stories/S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md` line 50

**Finding:**

Story `subsystems:` array at line 50 reads:

```yaml
subsystems: [SS-22, SS-17]
```

The story's `anchor_bcs:` array (line 23) includes BC-2.16.002. BC-2.16.002 file
frontmatter carries `subsystem: "SS-16"`. BC-INDEX confirms BC-2.16.002 maps to
SS-16 ("16 - Spec Engine"). ARCH-INDEX line 141 confirms SS-16 = "Spec Engine".

Additionally, AC-16 (MAX_REQUESTS_PER_PIPELINE) implements in
`crates/prism-spec-engine/src/pipeline.rs` — SS-16 territory.

**Sibling precedent:** S-PLUGIN-PREREQ-B (anchors BC-2.16.002) has
`subsystems: [SS-16, SS-01]` at its frontmatter line 36. The PREREQ-D story
anchors BC-2.16.002 but omits SS-16 — asymmetric with the PREREQ-B precedent.

**Why 26 prior passes missed this:**

POL-22 Phase A verified anchor existence and content (BC-2.16.002 §Catalog PASS).
It did NOT cross-check the story's `subsystems:` frontmatter array against the
`subsystem:` field of each anchored BC. This is a distinct verification axis:
frontmatter subsystem completeness vs anchor BC subsystem membership.

**Suggested fix (routed to story-writer for fix-burst-25):**

Update story frontmatter line 50 to `subsystems: [SS-22, SS-17, SS-16]` and add
a justification comment: `# SS-16 Spec Engine added: BC-2.16.002 (anchor) → subsystem SS-16 per BC-INDEX; AC-16 implements in prism-spec-engine`.

---

### F-LP27-MED-002 — PluginError enum lacks `#[non_exhaustive]` despite story adding 4 new variants (CLAUDE.md production-grade)

**Severity:** MEDIUM
**Confidence:** HIGH
**Category:** CLAUDE.md Canonical Principle Rule 1 violation (no MVP-driven deferrals);
Conventions §`#[non_exhaustive]` discipline

**Location:** Story line ~906 conditional MVP-hedge language + `crates/prism-core/src/error.rs:983` PluginError declaration

**Finding:**

`crates/prism-core/src/error.rs` lines 983-984:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PluginError {
```

NO `#[non_exhaustive]` attribute. The sibling `PrismError` at `error.rs` lines
15-17 reads:

```rust
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrismError {
```

This proves the established convention. CLAUDE.md Conventions: "All public
TOML-deserialized types and pub-API surface types require `#[non_exhaustive]`.
30+ types currently enforced via the compile-fail gate at
`tests/external/perimeter-violation/`."

The story (line ~906) uses conditional MVP-hedge language: "if PluginError is a
non-exhaustive enum" — hedging on whether the constraint applies. This is an
MVP-pattern hedge per CLAUDE.md Canonical Principle Rule 1 ("no MVP-driven
deferrals"). PluginError is a public enum in `prism-core`; it adds 4 new variants
in this story. The `#[non_exhaustive]` requirement is unconditional under the
established convention.

**Why 26 prior passes missed this:**

Prior adversarial focus targeted type contracts, error chains, and anchor content.
The `#[non_exhaustive]` attribute requirement for pub enums in `prism-core` was
not in the per-pass verification checklist. The MVP-hedge conditional language in
the story body normalizes the gap implicitly.

**Suggested fix (routed to story-writer for fix-burst-25):**

Strengthen the implementation prescription: PluginError MUST be marked
`#[non_exhaustive]` in the same commit as the new variants (per `PrismError`
precedent at `error.rs:15-17` and the 30+-type perimeter audit). Remove all
conditional "if PluginError is a non-exhaustive enum" language; replace with
unconditional requirement. The compile-fail gate at
`tests/external/perimeter-violation/` must be extended to cover `PluginError`
external match arms (wildcard `_ => {}` required for all external callers).

---

### F-LP27-MED-003 — §References section paraphrases 7 of 8 BC titles [process-gap, POL-7]

**Severity:** MEDIUM
**Confidence:** HIGH
**Category:** POL-7 violation (bc_h1_is_title_source_of_truth); process-gap sibling
to codification candidate #12

**Location:** `.factory/stories/S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md` lines 1007-1015

**Finding:**

The story §References section BC citation lines read as follows (paraphrased forms):

| Story Line | §References Text | Canonical BC H1 | Status |
|------------|-----------------|-----------------|--------|
| 1008 | `BC-2.17.001 — Plugin Panic Isolation` | `Plugin Panic Isolation — Crashed Plugin Does Not Terminate Host Process` | PARAPHRASE |
| 1009 | `BC-2.17.002 — Plugin Sandbox: No FS/Network Access` | `Plugin Sandbox — No Direct Filesystem or Network Access` | PARAPHRASE |
| 1010 | `BC-2.17.003 — Memory Limit 64MB` | `Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB)` | PARAPHRASE |
| 1011 | `BC-2.17.004 — CPU Time Limit via Epoch Interruption` | `Plugin Sandbox — CPU Time Limit Enforced via Epoch Interruption (default 5s)` | PARAPHRASE |
| 1012 | `BC-2.17.005 — Hot Reload Atomic Swap` | `Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version` | PARAPHRASE |
| 1013 | `BC-2.17.006 — WIT Validation Before Registration` | `WIT Interface Validation Before Plugin Registration` | PARAPHRASE |
| 1014 | `BC-2.17.007 — Plugin Manifest Schema Validation Before WIT Validation (NEW — landed ...)` | `Plugin Manifest Schema Validation Before WIT Validation` | VERBATIM (annotation acceptable) |
| 1015 | `BC-2.22.001 — Boot Orchestration Sequencing` | `Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate` | PARAPHRASE |

7 of 8 BC title citations in §References are paraphrased. POL-7
(`bc_h1_is_title_source_of_truth`) states: "All downstream references must match."
POL-7 scope explicitly includes `story`.

**Why 26 prior passes missed this:**

Codification candidate #12 (fix-burst-24 closure) applied BC title verbatim
verification to the story body BC table only. The §References section uses the
same BC identifiers but with independently authored paraphrase labels — a
distinct citation site that the body-BC-table sweep does not cover. The same
POL-7 principle applies; the gap survived because no pass had yet extended the
verbatim-check scope to §References.

This is the process-gap sibling to codification #12: codification #12 covered
the body BC table; the §References section needs the same treatment.

**Suggested fix (routed to story-writer for fix-burst-25):**

Rewrite §References section lines 1008-1015 to use verbatim BC H1 titles.
Preserve the BC-2.17.007 annotation pattern ("(NEW — landed ...)") as a
parenthetical after the verbatim title. Replace each other paraphrased title
with the canonical verbatim form per the table above.

**Codification candidate #13:** See Novel Class section below.

---

### F-LP27-LOW-001 — BC-2.17.005 cited at body lines 980 + §References line 1012 but missing from `inputs:` frontmatter

**Severity:** LOW
**Confidence:** HIGH
**Category:** Frontmatter completeness; sibling-site-sweep gap from fix-burst-23

**Location:** Story `inputs:` array (lines 53-62) vs body cites (line ~980 Architecture
Compliance Rules + §References line 1012)

**Finding:**

The story `inputs:` frontmatter array contains 8 BCs:
BC-2.16.002, BC-2.17.001, BC-2.17.002, BC-2.17.003, BC-2.17.004,
BC-2.17.006, BC-2.17.007, BC-2.22.001. BC-2.17.005 is absent.

However, BC-2.17.005 appears in two body locations:
1. Architecture Compliance Rules table (line ~980): `BC-2.17.005 §Invariants`
   (this citation was introduced by fix-burst-23, the spawn_blocking re-anchor)
2. §References section (line ~1012): `BC-2.17.005 — Hot Reload Atomic Swap`

This is a sibling-site-sweep gap from fix-burst-23: when spawn_blocking was
re-anchored to BC-2.17.005 §Invariants (F-LP25-HIGH-001), the `inputs:` array
was not updated to include BC-2.17.005. The frontmatter now references BC-2.17.005
implicitly (via Architecture Compliance Rules) without listing it in `inputs:`.

**Note:** BC-2.17.005 correctly remains absent from `behavioral_contracts:` and
`anchor_bcs:` arrays (the BC is cited as an architecture rule input, not as a
primary delivery target). The gap is in `inputs:` only.

**Suggested fix (routed to story-writer for fix-burst-25):**

Append `- ".factory/specs/behavioral-contracts/BC-2.17.005-plugin-hot-reload-atomic-swap.md"` to the `inputs:` array in story frontmatter.

---

### F-LP27-OBS-001 [process-gap] — POL-7 sweep scope insufficient: body BC table only; §References and other citation sites excluded

**Type:** OBSERVATION → 13th codification candidate
**Category:** Process-gap

**Description:**

Codification #12 (BC body-table title verbatim) was applied to the story body
BC table only. The same POL-7 "titles must match BC H1" principle applies to ALL
BC-title citation sites in a story:

- Body BC table (covered by codification #12)
- §References section (F-LP27-MED-003 — NOT covered)
- Frontmatter comments (not yet checked systematically)
- Architecture Mapping tables (not yet checked systematically)
- Architecture Compliance Rules prose (not yet checked systematically)
- Inline prose paragraphs citing BC titles (not yet checked systematically)

The §References section paraphrase class (7/8 titles) survived 26 passes
including the pass-26 codification #12 sweep, which by design targeted only the
body BC table rows. This demonstrates the incompleteness of the current sweep
scope.

**Proposed codification #13 — POL-22 Phase B Extension (cross-table BC title sweep):**

"For each BC-NNN.NNN identifier cited in a story (regardless of citation site:
body BC table, §References, Architecture Compliance Rules, frontmatter comments,
or prose), the accompanying title or description label MUST match the BC's H1
verbatim (whitespace-normalized). Paraphrasing is a POL-7 violation at any
citation site. Verification method: grep story for all BC-NNN.NNN patterns,
collect their surrounding label context, open each canonical BC file, compare
H1. Apply to all sites, not just the body BC table."

This extends POL-22 Phase B (codification #12) from body-BC-table-only scope
to full cross-table BC title coverage.

**Action:** Tag for cycle-close adjudication by session-reviewer. Possible upgrade
to formal POL-22 Phase B extension or POL-7 amendment with scope clarification.

---

## Novel Class — Codification Candidate #13

**Tag:** [process-gap] — POL-7 sweep must cover ALL BC title citation sites

**Class description:** Codification #12 (pass-26) established BC body-table
title verbatim verification for the body BC table. The §References section — a
distinct citation site in the same story — uses BC identifiers with independently
authored paraphrase labels that the body-BC-table sweep does not reach. The gap
is structural: the verification axis covers ONE citation form (body table) but not
the FULL citation population across the story.

**Evidence of gap:** F-LP27-MED-003 (7/8 §References BC titles paraphrased)
survived 26 passes including the pass-26 codification #12 sweep, because the
sweep correctly targeted only the body BC table. The §References section is a
sibling citation site with the same POL-7 requirement.

**Proposed Phase B extension (codification #13):**

Extend the BC-title-verbatim verification in POL-22 Phase B from body-BC-table
scope to full cross-document scope: verify ALL BC-NNN.NNN citations in a story
(body table, §References, Architecture Compliance Rules, frontmatter comments,
prose) carry verbatim BC H1 labels. The 13th recurrence of a new citation-site
class surviving the existing sweep methodology.

This is the **13th codification candidate**. Tag: `[process-gap]`.
Session-reviewer adjudicates at cycle-close.

---

## Summary

Pass-27 completed at story v1.24 (SHA 45ae2c2f) + BC-2.16.002 v1.12 (SHA 84f58565)
+ error-taxonomy v1.20 (SHA 8e980a0e).

Special verifications A, B, C all PASS: fix-burst-23's three in-scope fixes held
cleanly. Special Verification D PASS: fix-burst-24's BC-2.16.002 body-table title
closure also held — 8/8 BC body-table titles verbatim per codification #12.

POL-22 Phase A: 28/30 external anchors PASS (1 CONDITIONAL for SS-16 gap
= F-LP27-MED-001; 1 NOTE for BC-2.17.005 inputs gap = F-LP27-LOW-001).
POL-22 Phase B: 5/5 internal symmetry chains PASS (including codification #12
BC-title-verbatim body table chain).
Phase C carry-forward sample: 13/13 PASS, no regressions.

Phase D: 3 NEW MEDIUM findings + 1 LOW + 1 OBSERVATION (process-gap
codification candidate #13). All 3 MEDIUM findings represent finding classes
that 26 prior passes missed. F-LP27-MED-003 and F-LP27-OBS-001 are tagged
[process-gap] as codification #13 candidate.

**Verdict: BLOCKED** per BC-5.39.001. Streak holds 0/3.

**Next action:** Dispatch story-writer for fix-burst-25 (3 MED + 1 LOW in-scope;
F-LP27-OBS-001 tagged for cycle-close adjudication):
- F-LP27-MED-001: add SS-16 to `subsystems:` frontmatter + justification comment
- F-LP27-MED-002: strengthen PluginError `#[non_exhaustive]` prescription, remove MVP-hedge conditional language
- F-LP27-MED-003: rewrite §References lines 1008-1015 with verbatim BC H1 titles
- F-LP27-LOW-001: append BC-2.17.005 to `inputs:` frontmatter array

Pass-28 after fix-burst-25 closure.

Producer: adversary (vsdd-factory). Reified by state-manager (27th consecutive
adversary reification by state-manager — formal codification confirmed).
