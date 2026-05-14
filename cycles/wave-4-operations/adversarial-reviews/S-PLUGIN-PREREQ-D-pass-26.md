---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 26
target_sha: 55170313
story_content_sha: 2b2157f7
error_taxonomy_content_sha: 8e980a0e
bc_content_sha: 84f58565
base_sha: 95d46be2
verdict: BLOCKED
streak: "0/3 HOLD (pass-26 BLOCKED: 1 MED — asymmetric BC table title)"
finding_summary: {CRITICAL: 0, HIGH: 0, MEDIUM: 1, LOW: 0, OBS: 1}
prior_passes: [pass-1, pass-2, pass-3, pass-4, pass-5, pass-6, pass-7, pass-8, pass-9, pass-10, pass-11, pass-12, pass-13, pass-14, pass-15, pass-16, pass-17, pass-18, pass-19, pass-20, pass-21, pass-22, pass-23, pass-24, pass-25]
prior_fix_bursts: [fix-burst-1, fix-burst-2, fix-burst-3, fix-burst-4, fix-burst-5, fix-burst-6, fix-burst-7, fix-burst-8, fix-burst-9, fix-burst-10, fix-burst-11, fix-burst-12, fix-burst-13, fix-burst-14, fix-burst-15, fix-burst-16, fix-burst-17, fix-burst-18, fix-burst-19, fix-burst-20, fix-burst-21, fix-burst-22, fix-burst-23]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1"
idempotency_check: false
post_fix_check: true
post_fix_target: "fix-burst-23 (F-LP25-HIGH-001 spawn_blocking re-anchor + F-LP25-LOW-001 SS-17 + F-LP25-LOW-002 AC-9 HTTP defaults strip)"
producer: "adversary (vsdd-factory; reified by state-manager due to read-only tool profile)"
---

# Adversarial Pass 26 — S-PLUGIN-PREREQ-D

**Verdict: BLOCKED (1 MEDIUM + 1 OBSERVATION)**

**Context:** This is a post-fix-burst-23 fresh-context pass. Fix-burst-23 closed
F-LP25-HIGH-001 (spawn_blocking re-anchor), F-LP25-LOW-001 (SS-17 short-name),
and F-LP25-LOW-002 (AC-9 fabricated hedge). The expected outcome was CLEAN (0/3 → 1/3).
Actual outcome: BLOCKED by 1 MEDIUM finding — a class that 25 prior passes missed.
Streak holds at 0/3 per BC-5.39.001.

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

## POL-22 Phase A — External Anchor Verification (25-Anchor Table)

All 25 external anchors verified via fresh-context grep of cited target documents.
Story-body substring presence alone is NOT sufficient (process-gap codification
candidate #11 — adversary must open and grep cited documents).

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

**POL-22 Phase A result: 25/25 PASS**

---

## POL-22 Phase B — Internal Cross-Reference Symmetry Chains (4-Chain Table)

| Chain | Chain Elements | Result |
|-------|---------------|--------|
| Vec<String> contract chain | AC-7 declaration → AC-17 body `Vec<String>` → Task 2 reference → `test_default()` `Vec<String>` → all 6 Match-Site rows `Vec<String>` | PASS |
| E-PLUGIN-013/014/015/016 four-layer | AC-5 error reference → Error Taxonomy Additions §E-PLUGIN-013..016 rows → BC-2.17.004 §Invariants → prism-spec-engine error.rs canonical | PASS |
| E-PIPELINE-001 five-layer | AC-16 rate-limit → Error Taxonomy Additions §E-PIPELINE-001 row → error-taxonomy.md PIPELINE namespace → BC-2.16.002 §preconditions MAX_REQUESTS cap → `SpecEngineError::TooManyRequests` canonical type | PASS |
| Manifest four-code symmetry | AC-3 manifest fields → §Structured Event Catalog Additions 9 rows → BC-2.16.002 §Catalog entries → wasm-manifest.toml schema | PASS |

**POL-22 Phase B result: 4/4 PASS**

---

## Phase C — Carry-Forward Sample (9-Sample Table)

9 representative closures from prior passes verified for regression.

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

**Phase C carry-forward sample: 9/9 PASS — no regressions.**

---

## Phase D — New Findings

### F-LP26-MED-001 — BC-2.16.002 Body BC Table Title Differs from Canonical BC H1 + BC-INDEX [process-gap]

**Severity:** MEDIUM
**Confidence:** HIGH
**Category:** POL-7 violation (bc_h1_is_title_source_of_truth); POL-4 violation (semantic_anchoring_integrity)

**Location:** Story body Behavioral Contracts table, BC-2.16.002 row (story line 254).

**Finding:**

Story body Behavioral Contracts table row for BC-2.16.002 reads:

```
| BC-2.16.002 | Multi-Step Fetch Pipeline — Structured Event Catalog | AC-16 ... |
```

The canonical BC H1 (`.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md` line 30) reads:

```
# BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation
```

The BC-INDEX entry for BC-2.16.002 reads:

```
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | ...
```

**Asymmetry:** The story body BC table contains 8 total BC rows. The other 7
(BC-2.17.001/002/003/004/006/007 + BC-2.22.001) all carry Title cells that match
their respective BC H1 verbatim. Only BC-2.16.002 deviates.

The deviation has two distinct errors relative to the canonical title:
1. "Execution" is dropped after "Pipeline"
2. "Structured Event Catalog" is substituted for the canonical subtitle "Sequential Steps with Variable Interpolation"

The substituted phrase "Structured Event Catalog" correctly describes the scope of
the story's contribution to BC-2.16.002 (adding 9 new event_type rows to the Catalog),
but it belongs in the Primary Coverage column, not the Title cell. The Title cell must
match BC H1 verbatim per POL-7.

**Why 25 prior passes missed this:**

POL-22 Phase B (internal cross-reference symmetry chains) verified that BC row entries
were internally consistent (type contracts, error chains) and that the referenced BCs
existed. It did NOT specifically verify that EACH BC body-table Title cell matches its
BC H1 verbatim. This is a distinct verification axis: cross-document title fidelity
at the per-row level.

**Suggested fix (routed to story-writer for fix-burst-24):**

Replace the BC-2.16.002 Title cell (line 254) with the verbatim canonical:

```
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | AC-16 (MAX_REQUESTS_PER_PIPELINE cap; traces to BC-2.16.002 preconditions); Structured Event Catalog Additions §intro (9 new event_type rows) |
```

The story-specific sub-scope description ("Structured Event Catalog Additions §intro")
is preserved in the Primary Coverage column where it semantically belongs.

**Codification candidate #12:** See Novel Class section below.

---

### OBS-LP26-001 — VP-PLUGIN-007 "not-None" Description Stale Under Vec<String> Contract

**Type:** OBSERVATION (not a blocker)
**Location:** `.factory/specs/verification-properties/VP-INDEX.md` lines 174 + 190

**Context:** VP-PLUGIN-007's description still uses "not-None" framing, which was
accurate under the original `Option<Vec<String>>` type contract. The type was changed
to `Vec<String>` (empty-list semantics) by fix-burst-17. The "not-None" framing is
semantically incoherent under the new contract.

**Prior routing:** This observation corresponds to F-LP19-LOW-002, which was already
routed to phase-5 deferred at story v1.23 changelog row 1029 cross-reference. The
routing was acknowledged in the D-513 Burst A notes.

**Status:** Already deferred to phase-5 via prior F-LP19-LOW-002. No action required
in this burst. Recorded here for completeness and continuity of the pass record.

**No action this burst.**

---

## Novel Class — Codification Candidate #12

**Tag:** [process-gap] — BC body-table title verbatim verification

**Class description:** Prior adversarial discipline (POL-22 Phase B) verified internal
type-unification chains across multiple BC rows within a story's body table, confirming
that type contracts, error chains, and cross-reference symmetry were internally coherent.
It did NOT specifically verify that EACH BC body-table Title cell matches its canonical
BC H1/BC-INDEX verbatim.

**Evidence of gap:** The asymmetry between BC-2.16.002 (paraphrased) and the other
7 BCs in the same body table (verbatim) survived 25 passes — including the idempotency
check at pass-25. The paraphrase was plausible enough to avoid triggering either
Phase A (external anchor existence) or Phase B (internal symmetry chain) checks.

**Proposed Phase B extension:**

"For each BC row in a story's Behavioral Contracts body table, the Title cell MUST
match the BC's H1 exactly (whitespace-normalized). Substantive paraphrasing (e.g.,
substituting a sub-scope label for the verbatim subtitle) is a POL-7 violation and
a MEDIUM finding. Verification method: for each BC-NNN.NNN in the table, open the
canonical BC file and compare `# BC-NNN.NNN: <title>` against the Title cell in the
story body table."

This is the **12th codification candidate**. Tag: `[process-gap]`.
Session-reviewer adjudicates at cycle-close.

---

## Summary

Pass-26 completed at story v1.23 (SHA 2b2157f7) + BC-2.16.002 v1.12 (SHA 84f58565)
+ error-taxonomy v1.20 (SHA 8e980a0e).

Special verifications A, B, C all PASS: fix-burst-23's three in-scope fixes held cleanly.

POL-22 Phase A: 25/25 external anchors PASS.
POL-22 Phase B: 4/4 internal symmetry chains PASS.
Phase C carry-forward sample: 9/9 PASS, no regressions.

Phase D: 1 NEW MEDIUM finding (F-LP26-MED-001) + 1 OBSERVATION already routed
(OBS-LP26-001).

**Verdict: BLOCKED** per BC-5.39.001. Streak holds 0/3.

**Next action:** Dispatch story-writer for fix-burst-24 (single in-scope fix:
BC-2.16.002 Title cell at line 254, replace with verbatim canonical BC H1).
Pass-27 after fix-burst-24 closure.

Producer: adversary (vsdd-factory). Reified by state-manager (26th consecutive
adversary reification by state-manager — formal codification confirmed).
