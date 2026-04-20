---
document_type: adversarial-review
level: ops
version: "1.0"
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[live-state]"
traces_to: prd.md
cycle: phase-2-patch
pass: 37
previous_review: pass-36.md
status: findings-open
novelty: MEDIUM — HIGH-001 is fresh-context 4-row systematic title drift in a single story that prior passes' AC-trace focus did not audit; MED-001 surfaces frontmatter ownership-vs-reference ambiguity; OBS-001 is lint-automation forward concern
findings_total: 3
findings_crit: 0
findings_high: 1
findings_med: 1
findings_low: 0
findings_observational: 1
previous_pass: 36
convergence_counter: 0
date: 2026-04-19
---

# Adversarial Review: Prism (Pass 37)

## Finding ID Convention

`P3P37-A-{SEV}-NNN` where SEV is CRIT / HIGH / MED / LOW.

## Part A — Methodology

### Dimensions Scanned (12)

1. Semantic anchoring integrity (Policy 4) — BC-ID / error-code / tool-name alignment across story bodies
2. Changelog discipline (Policy 2) — version bumps, changelog completeness, inventory currency
3. Arithmetic consistency (Policy 6 adjacent) — count claims in Mermaid labels, frontmatter totals
4. Policy 8 bidirectional AC-to-BC trace — acceptance-criteria ↔ BC-INDEX cross-reference integrity
5. BC-INDEX ↔ story-body title sync — canonical H1 titles propagated to story BC tables (Policy 7)
6. Error taxonomy propagation — new error codes from Burst 36 reflected in story bodies + BCs
7. Capability enumeration completeness — CAP-NNN tool lists vs api-surface.md + story ACs
8. SS-anchor correctness — SS-ID assignments match architecture subsystem names
9. VP-INDEX ↔ architecture traceability — VP references in stories trace to declared VPs
10. Test-vector consistency — test-vectors.md scenario error codes / tool names match canonical sources
11. Cross-document version pin integrity — supplement pinned versions match current file versions
12. STORY-INDEX BC traceability matrix — frontmatter BC arrays reflected in matrix ownership rows

### Policies Applied

- policies.yaml rubric (full 9-policy set)
- Policy 7 (bc_h1_is_title_source_of_truth) — primary axis for H-001
- Policy 8 spirit (bc_array_changes_propagate_to_body_and_acs) — primary axis for MED-001

### Corpus

| Artifact | Version | Lines |
|----------|---------|-------|
| BC-INDEX | v4.10 | 203 BCs (195 active + 6 dual-anchor + 2 removed) |
| STORY-INDEX | v1.25 | 75 stories |
| ARCH-INDEX | current | — |
| capabilities.md | v1.2 | — |
| api-surface.md | v1.2 | — |
| error-taxonomy.md | v1.2 | — |
| VP-INDEX | v1.3 | 39 VPs (20+11+6+2) |
| test-vectors.md | v2.2 | — |

---

## Part B — New Findings

### P3P37-A-HIGH-001 — S-5.06 body BC Table titles drift from BC-INDEX canonical (Policy 7)

**Severity:** HIGH
**Policy:** 7 (bc_h1_is_title_source_of_truth)
**Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-5.06-action-infusion-tools.md` lines 50–55

**Description:**

The Behavioral Contracts table in S-5.06's body uses paraphrased or truncated titles for all 4 frontmatter BCs. BC-INDEX v4.10 defines authoritative H1 titles and Policy 7 requires verbatim propagation. Mapping of drift:

| BC-ID | S-5.06 body title (current) | BC-INDEX canonical title (required) |
|-------|----------------------------|--------------------------------------|
| BC-2.18.003 | `Action Manual Fire-and-Forget` | `Manual Action Triggers — Fire-and-Forget, Result Returned Immediately to AI Caller` |
| BC-2.17.005 | `Plugin Hot Reload — Atomic Module Swap` | `Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version` |
| BC-2.19.004 | `Infusion Hot Reload Atomicity` | `Infusion Hot Reload — Failed Validation Retains Previous Registration (CI-002)` |
| BC-2.05.001 | `Audit Entry Per Tool Invocation` | `Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes)` |

Prior passes audited AC-trace bidirectional alignment for these BCs but did not audit the BC table title strings themselves in S-5.06 body. This 4-row drift was not visible in the AC-trace dimension.

**Fix:** Replace all 4 title strings in S-5.06 body BC table verbatim from BC-INDEX v4.10. No structural changes needed — title column values only.

---

### P3P37-A-MED-001 — STORY-INDEX BC-traceability matrix missing S-5.06 mappings for 4 BCs

**Severity:** MED
**Policy:** 8 spirit (bc_array_changes_propagate_to_body_and_acs — STORY-INDEX should reflect frontmatter)
**Location:** `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` lines 154 (Full Story List), 225, 356, 360, 370 (BC Traceability Matrix)

**Description:**

S-5.06 frontmatter declares 4 BCs with 4 AC traces:

```yaml
behavioral_contracts:
  - BC-2.18.003
  - BC-2.17.005
  - BC-2.19.004
  - BC-2.05.001
```

However:

- STORY-INDEX Full Story List row 154 shows BCs=0 for S-5.06
- BC Traceability Matrix rows 225/356/360/370 list only peer stories (S-2.04/S-5.10/S-1.15/S-4.08/S-1.14) as owners for these 4 BCs — S-5.06 is absent from all four rows

**Ambiguity noted:** S-5.06 body (lines 46-48) frames BCs as "consumed via cross-subsystem references" (not owned), but frontmatter lists them in the same `behavioral_contracts:` field used for ownership. No convention document disambiguates ownership-vs-reference semantics in the frontmatter field.

**Fix direction (chosen): Option (b)** — Update STORY-INDEX:

1. Full Story List S-5.06 row: BCs count 0 → 4
2. BC Traceability Matrix: add S-5.06 as co-owner to rows for BC-2.05.001, BC-2.17.005, BC-2.18.003, BC-2.19.004

Rationale: S-5.06 becomes a co-owner because its ACs test those behaviors via the MCP tool invocation surface (the story exercises the BCs, regardless of subsystem origin). Bump STORY-INDEX v1.25 → v1.26 with changelog entry.

---

### P3P37-A-OBS-001 — "behavioral_contracts" vs "bcs" field-name convention mismatch

**Severity:** OBSERVATIONAL
**Policy:** 8 (forward lint concern only)
**Location:** All 75 stories use `behavioral_contracts:` frontmatter; policies.yaml Policy 8 (lines 120-132) refers to `bcs:` throughout.

**Description:**

All 75 stories use `behavioral_contracts:` as the frontmatter key for BC arrays. policies.yaml Policy 8 refers to this field as `bcs:` throughout its text. This creates a latent risk: any future auto-validator keyed to the literal string `bcs:` would silently pass drift that should be flagged.

This is observational only; it does not block convergence. No story drift is introduced by this mismatch — the field is read and applied correctly by all current human and agent consumers.

**Fix direction:** Align one side:

- Option A: Rename story frontmatter field `behavioral_contracts:` → `bcs:` — breaking migration across 75 files, high friction
- Option B: Update policies.yaml Policy 8 text to reference `behavioral_contracts:` — low friction, no story file changes

Option B is preferred. **Deferred to post-convergence unless user directs otherwise.**

---

## Summary

| Severity | Count | Finding IDs |
|----------|-------|-------------|
| CRIT | 0 | — |
| HIGH | 1 | P3P37-A-HIGH-001 |
| MED | 1 | P3P37-A-MED-001 |
| LOW | 0 | — |
| OBS | 1 | P3P37-A-OBS-001 |
| **Total** | **3** | |

**Verdict: NOT CLEAN.** 3 findings open (1 HIGH, 1 MED, 1 OBS). Convergence counter stays at 0/3.

Burst 38 must close:

- P3P37-A-HIGH-001 (S-5.06 lines 50–55: replace 4 BC table titles verbatim from BC-INDEX v4.10)
- P3P37-A-MED-001 (STORY-INDEX v1.25 → v1.26: Full Story List BCs count 0→4 + matrix co-ownership for 4 BCs)
- P3P37-A-OBS-001 (deferred — policies.yaml Policy 8 field-name alignment; post-convergence)

### Sweeps Run Clean

- VP-INDEX arithmetic (20+11+6+2=39; P0=32, P1=7) ✓
- BC-INDEX arithmetic (195+6+2=203) ✓
- Burst 37 verifications: S-5.06:199 E-ACTION-006 ✓; S-1.15:365 parenthetical ✓; api-surface.md:51 "24 Write Tools" ✓; api-surface.md:24 "28 Read Tools" ✓; v1.3 changelog ✓
- Tool registry row counts (28 always-visible, 24 capability-gated) ✓
- S-5.06, S-1.15 AC-trace bidirectional Policy 8 ✓
- S-1.15 body BC table titles match BC-INDEX ✓
- S-4.08 body BC table title for BC-2.18.003 ✓
- No residual `execute_action` refs in S-5.06 ✓
- ARCH-INDEX SS-01..SS-20 ✓
- BC-2.17.005 SS-17 / CAP-030 frontmatter ✓
- error-taxonomy contains E-ACTION-006 and E-PLUGIN-003 ✓
