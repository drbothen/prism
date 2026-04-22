---
document_type: adversarial-review
level: ops
version: "1.0"
status: findings-open
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
pass: 31
previous_review: pass-30.md
cycle: phase-2-patch
novelty: MEDIUM
findings: 6
critical: 0
high: 1
medium: 4
low: 1
previous_pass: 30 (4 findings: 3 MED, 1 LOW — all 4 closed Burst 31)
convergence_counter: 0 of 3
---

# Pass 31 — Burst 31 closures verified; first comprehensive Policy 8 sweep surfaces 6 systematic gaps + S-1.05 Task 6 propagation miss

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix from `.factory/current-cycle` — `P3PATCH` for phase-2-patch
- `<PASS>`: Two-digit pass number (`P31`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass

This cycle uses short-form IDs (`P3P31-A-H-001` etc.) consistent with all prior passes in this cycle.

## Scope

Fresh-context review at commit `538963e`. Primary focus: Burst 31 closure verification (4 items), first comprehensive Policy 8 bidirectional sweep across all 73 stories (not just the 2 pass-30 sampled). Also: scripted BC-title sweep re-verification, BC-INDEX/VP-INDEX arithmetic, Policy 2 orphan scan, Policy 6 ARCH-INDEX sync, test-vectors.md v2.1 structural integrity.

## Part A — Fix Verification (Burst 31 Closure)

| Finding | Claim | Verified |
|---------|-------|----------|
| M-001 | S-1.05 line 51 "Four-tier" | CLOSED — verbatim match |
| M-002 | S-1.10 +AC-6/7/8 for BC-2.09.001/.006/.007 | CLOSED — all 3 ACs land with correct BC postcondition wording |
| M-003 | S-1.08 +AC-8 for BC-2.04.003 | CLOSED — AC-8 cites most-specific-wins + Deny-override scenario |
| L-001 | S-1.10 Task 4 centralized-array | CLOSED — "NO per-field `{field}_safety_flag` parallel fields" |

All 4 pass-30 items landed; no regressions.

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

#### P3P31-A-H-001 — Systematic Policy 8 AC-trace gap across 6 stories (13 BC-level gaps) — first comprehensive sweep this cycle

- **Severity:** HIGH (Lessons Learned: "Systematic pattern across 3+ stories: HIGH with pattern flag" — 6 stories)
- **Category:** ac-coverage
- **Policy violated:** 8 (`bc_array_changes_propagate_to_body_and_acs`)
- **Location:** S-6.04, S-5.07, S-4.08, S-1.15, S-1.09, S-2.04
- **Novelty:** NEW — pass-30 only sampled S-1.10 and S-1.08; this is the first full 73-story sweep
- **Description:** First comprehensive bidirectional Policy 8 sweep this cycle surfaces 13 BC-level AC-trace gaps across 6 stories. Each BC appears in frontmatter `behavioral_contracts:` AND body BC table, but no AC line traces to it.
- **Evidence:**

| Story | BCs missing AC trace | Count |
|-------|---------------------|-------|
| S-6.04-credential-cli.md | BC-2.03.002, .003, .004, .005, .010 | 5 |
| S-5.07-multi-repo-git-config.md | BC-2.06.002, .007, .010 | 3 |
| S-4.08-action-delivery.md | BC-2.18.003, .008 | 2 |
| S-1.15-wasm-runtime.md | BC-2.17.003 | 1 |
| S-1.09-confirmation-tokens.md | BC-2.04.007 | 1 |
| S-2.04-audit-construction.md | BC-2.05.006 | 1 |

Checked both `(traces to BC-X.XX.YYY ...)` and parenthetical `(BC-X.XX.YYY)` citations, plus INV-<ALIAS>-NNN alternates where applicable.

- **Why pass-30 missed this:** Pass-30 Policy 8 check was scoped to S-1.10 and S-1.08 only; comprehensive sweep was deferred until pass-31 had fresh context.
- **Proposed Fix:** Add 13 ACs across the 6 stories. Each new AC must match an actual BC postcondition and use canonical `(traces to BC-X.XX.YYY ...)` format. Read each BC file before writing ACs.

---

### MEDIUM

#### P3P31-A-M-101 — S-1.05 Task 6 still says "three-tier" contradicting fixed line 51 and BC-2.02.008 body

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Policy violated:** 4 (semantic_anchoring_integrity), 7 (bc_h1_is_title_source_of_truth)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.05-ocsf-field-mapping.md` lines 164-173
- **Novelty:** NEW — propagation miss from Burst 31 M-001 fix (scoped narrowly to line 51)
- **Description:** Burst 31 fixed line 51 description cell to "Four-tier" but Task 6 body prose still describes a 3-tier model with completely wrong tier semantics. AC-8 also tests the 3-tier model. Implementer building from Task 6 will miss the Prism-metadata tier and produce a non-conforming resolver.
- **Evidence:**
  - Line 51 (FIXED): `Four-tier field alias resolution: Prism metadata → Proto descriptor fields → raw_extensions JSON → None` ✓
  - Task 6 (lines 164-168) still says: "Implement three-tier field alias resolution in crates/prism-ocsf/src/alias.rs (BC-2.02.008): Tier 1 (OCSF canonical): direct OCSF field name match; Tier 2 (vendor alias): per-sensor alias table; Tier 3 (fallback): if neither matches, field goes to extensions"
  - AC-8 (lines 226-228) tests the 3-tier model, not the 4-tier BC model
  - BC-2.02.008 body postconditions enumerate FOUR tiers: (1) Prism metadata, (2) Proto descriptor fields, (3) raw_extensions JSON, (4) None
- **Proposed Fix:** Rewrite Task 6 with BC-2.02.008's 4 tiers:
  - Tier 1: Prism-specific metadata fields (`source_sensor`, `source_record_type`, `client_id`)
  - Tier 2: Proto descriptor fields via recursive descent into DynamicMessage (dot notation `device.hostname`)
  - Tier 3: Unmapped JSON fields from raw_extensions
  - Tier 4: None (field absent)

  Also update AC-8 to reflect the correct BC-2.02.008 tier model.

---

#### P3P31-A-M-102 — S-6.04 5 BC-level AC-trace gaps (breakdown of H-001)

- **Severity:** MEDIUM
- **Category:** ac-coverage
- **Policy violated:** 8 (bc_array_changes_propagate_to_body_and_acs)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-6.04-credential-cli.md`
- **Novelty:** See H-001
- **Description:** Per-story remediation breakdown for auditability. S-6.04 has 5 BCs in frontmatter/body with no AC trace: BC-2.03.002, .003, .004, .005, .010.
- **Evidence:** See H-001 evidence table.
- **Proposed Fix:** Add 5 ACs (one per BC). Read BC-2.03.002/.003/.004/.005/.010 files before writing.

---

#### P3P31-A-M-103 — S-5.07 + S-4.08 BC-level AC-trace gaps (breakdown of H-001)

- **Severity:** MEDIUM
- **Category:** ac-coverage
- **Policy violated:** 8 (bc_array_changes_propagate_to_body_and_acs)
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-5.07-multi-repo-git-config.md` (3 gaps); `/Users/jmagady/Dev/prism/.factory/stories/S-4.08-action-delivery.md` (2 gaps)
- **Novelty:** See H-001
- **Description:** Per-story remediation breakdown. S-5.07 missing AC traces for BC-2.06.002, .007, .010. S-4.08 missing AC traces for BC-2.18.003, .008.
- **Evidence:** See H-001 evidence table.
- **Proposed Fix:** S-5.07 +3 ACs; S-4.08 +2 AC trace updates.

---

#### P3P31-A-M-104 — S-1.15, S-1.09, S-2.04 single BC-level AC-trace gaps (breakdown of H-001)

- **Severity:** MEDIUM
- **Category:** ac-coverage
- **Policy violated:** 8 (bc_array_changes_propagate_to_body_and_acs)
- **Location:** S-1.15 (BC-2.17.003 memory limit), S-1.09 (BC-2.04.007 risk-tier classification), S-2.04 (BC-2.05.006 append-only immutability)
- **Novelty:** See H-001
- **Description:** Per-story remediation breakdown. Three stories each missing 1 AC trace.
- **Evidence:** See H-001 evidence table.
- **Proposed Fix:** +1 AC each across S-1.15, S-1.09, S-2.04.

---

### LOW

#### P3P31-A-L-201 — AC-trace citation style drift across 73 stories (observational only)

- **Severity:** LOW
- **Category:** spec-fidelity
- **Policy violated:** 4 (loose — mechanical tooling friction)
- **Location:** Corpus-wide
- **Novelty:** OBSERVATIONAL
- **Description:** Three distinct AC-trace citation styles coexist across stories, creating tooling friction for consistency-validators that grep for BC-ID literals.
- **Evidence:**
  1. Explicit `(traces to BC-X.XX.YYY postcondition N)` — S-5.05, S-5.04, S-1.10 post-Burst-31 (most machine-parseable)
  2. Parenthetical `(BC-X.XX.YYY)` — S-3.02, S-4.01/02/03/06 (legible but less explicit)
  3. INV-<ALIAS>-NNN without direct BC-ID — S-4.08, S-1.15, S-1.14 (maps 1:1 within subsystem but requires alias lookup)
- **Proposed Fix:** Standardize on style 1 long-term OR maintain canonical alias-to-BC lookup. Post-v1.0 quality of life; defer for now.

---

## Observations

1. Burst 31 surgical fixes all landed cleanly.
2. BC-INDEX v4.10 arithmetic exact: 195 + 6 + 2 = 203 ✓; per-subsystem column sums (9+12+12+15+11+10+6+9+8+11+15+10+14+12+11+10+6+9+5+0 = 195); P0 166 + P1 29 = 195.
3. VP-INDEX arithmetic exact: 39 total (20+11+6+2). verification-architecture + coverage-matrix all consistent.
4. All 28 active DIs cited by ≥1 BC — Policy 2 clean.
5. test-vectors.md v2.1 structural integrity clean.
6. ARCH-INDEX SS-01..SS-20 stable; no renames.
7. Scripted BC-title sweep (pass-30 methodology) still shows 0 drifts in canonical 2-col stories.
8. S-1.05 Task 6 contradiction validates "Fresh-Context Compounding Value" — narrow line-51 fix missed Task 6 propagation.

## Novelty Assessment

**NOVELTY: MEDIUM.** H-001 (systematic Policy 8 sweep) is genuinely novel — first full 73-story bidirectional audit. M-101 (S-1.05 Task 6) is novel — survived narrowly-scoped Burst 31 fix. Neither is refinement of pass-30 findings.

Trajectory: 26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→5→5→4→**6**. Uptick due to FIRST comprehensive Policy 8 sweep finding 13 gaps. CRIT=0 streak holds (19+ passes). HIGH=0 from pass-30 briefly interrupted by H-001 pattern finding.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 4 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate
**Readiness:** requires revision

**Burst 32 scope (surgical):**
1. M-101: S-1.05 Task 6 rewrite + AC-8 update to 4-tier model
2. H-001: Add 13 ACs across 6 stories (S-6.04 +5, S-5.07 +3, S-4.08 +2, S-1.15/1.09/2.04 +1 each)
3. L-201: Defer (observational, post-v1.0)

Pass-32 can advance convergence counter to 1/3 if Burst 32 closes cleanly.

## Relevant Files

**Burst 31 closure (verified):**
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.05-ocsf-field-mapping.md` (line 51 FIXED; Task 6 M-101)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.10-prompt-injection-defense.md`
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.08-feature-flags.md`

**H-001 Policy 8 gaps:**
- `/Users/jmagady/Dev/prism/.factory/stories/S-6.04-credential-cli.md` (5 gaps)
- `/Users/jmagady/Dev/prism/.factory/stories/S-5.07-multi-repo-git-config.md` (3 gaps)
- `/Users/jmagady/Dev/prism/.factory/stories/S-4.08-action-delivery.md` (2 gaps)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.15-wasm-runtime.md` (1 gap)
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.09-confirmation-tokens.md` (1 gap)
- `/Users/jmagady/Dev/prism/.factory/stories/S-2.04-audit-construction.md` (1 gap)

**Source-of-truth references:**
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.02.008-field-alias-resolution.md` (4-tier SoT for M-101)
- BC files for each H-001 gap (read before writing ACs)
