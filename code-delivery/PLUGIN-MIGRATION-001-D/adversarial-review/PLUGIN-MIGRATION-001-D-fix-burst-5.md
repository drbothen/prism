---
document_type: fix-burst-closure-record
story_id: PLUGIN-MIGRATION-001-D
fix_burst_number: 5
pass_addressed: 5
closure_date: 2026-05-20
closure_decision: D-739
streak_status: 0/3 (unchanged; awaiting pass-6 fresh-context)
findings_total: 6
findings_closed: 5
findings_deferred: 1
deferred_items: [F-LP5-OBS-001 process-gap — deferred to S-7.02 cycle-close codification (ADR-internal cross-row consistency check for source-of-truth grounding ADRs)]
---

# PLUGIN-MIGRATION-001-D Fix-Burst-5 Closure

## Findings Closure Status

### HIGH (1/1 closed in-scope)

| Finding | Closure | Evidence |
|---|---|---|
| F-LP5-HIGH-001 (ADR-028 §D2 Armis row declares `auth_type = "api_key"` — contradicts BC, story, HS which all declare `bearer_static`) | CLOSED | ADR-028 v1.0 → v1.1: §D2 Armis row corrected from `api_key` to `bearer_static`; cross-checked vs `crates/prism-dtu-armis/src/lib.rs:16-17` BearerStatic enforcement. The legacy `ArmisAuth::auth_type_name()` returning `"api_key"` is itself the latent label bug §D2 was authored to immunize against. Architect fix (dispatch 1 of 2). ARCH-INDEX v2.86 → v2.87. |

### MED (2/2 closed in-scope)

| Finding | Closure |
|---|---|
| F-LP5-MED-001 (STORY-INDEX `crates_touched` lists 3 crates; story frontmatter lists 2 — POL-13 violation) | CLOSED — Story v1.4 → v1.5 frontmatter `crates_touched: [prism-sensors, prism-spec-engine, prism-core]` (prism-core added per Task 11 scope expansion D-737 Decision 3); STORY-INDEX v2.161 → v2.162 confirms 3-crate agreement. Story-writer fix. |
| F-LP5-MED-002 (BC-2.16.001 v1.5 `modified: null` contradicts Changelog v1.5 entry dated 2026-05-20 — POL-27 violation) | CLOSED — BC-2.16.001 frontmatter `modified: "2026-05-20"` set. BC-2.16.009 caught by PO POL-29 sibling sweep as bonus closure: `modified: "2026-05-20"` set (BC-2.16.009 was in same modified-field sweep, not in adversary's explicit finding list). Product-owner fix. |

### LOW (2/2 closed in-scope)

| Finding | Closure |
|---|---|
| F-LP5-LOW-001 (Cyberint cookie citation off-by-1: "lines 43-46" in BC, ADR-028, HS-015, story — replace with symbol anchor `::extract_session_token()` per TD-VSDD-091) | CLOSED — POL-25 multi-cite symbol-anchor sweep: BC-2.16.013 v1.4 → v1.5 (cyberint `extract_session_token` symbol anchor at §Postconditions Cyberint row); HS-015 v1.1 → v1.2 (2 citation sites updated); ADR-028 v1.1 → v1.2 (§D2 cyberint cite symbol-anchored to `::extract_session_token()`; ARCH-INDEX v2.87 → v2.88); story v1.4 → v1.5 body propagation at AC-011 + Task 5 (cyberint symbol anchor propagated). Product-owner + architect (dispatch 2 of 2) + story-writer joint fix. |
| F-LP5-LOW-002 (Story Task 11 conditional phrasing "check before… if not marked" — answerable in spec scope; production-grade default Rule 6 violation) | CLOSED — Story Task 11 rewritten unconditional: "Add `#[non_exhaustive]` to `SpecErrorCode` enum" + investigation result explicitly stated: SpecErrorCode is NOT in the non-exhaustive-violation compile-fail gate scope (EXPECTED stays 32; gate covers external-crate pub-API surface, not internal error enums). Story-writer fix. |

### OBS (1 — deferred, out-of-scope process-gap)

| Finding | Disposition |
|---|---|
| F-LP5-OBS-001 [process-gap] (ADR authoring of source-of-truth grounding rules lacks internal cross-row self-consistency check — F-LP5-HIGH-001 is same class as the bug §D2 was authored to prevent) | DEFERRED to S-7.02 cycle-close codification per orchestrator routing. Required codification: architect prompts and consistency-validator dispatch criteria for ADRs anchoring parallel rows must include a cross-row self-consistency check step. Not blocking this cascade. |

## Cumulative Closures (All 5 Fix-Bursts)

| Burst | Pass Addressed | Findings Closed | Severity Breakdown |
|---|---|---|---|
| FB-IMPL-P1 (D-733) | Pass 1 | 14 | 5H + 3M + 4L + 2OBS |
| FB-IMPL-P2 (D-734) | Pass 2 | 10 | 3H + 3M + 2L + 2OBS |
| FB-IMPL-P3 (D-735) | Pass 3 | 12 | 3C + 2H + 1M + 6OBS |
| FB-IMPL-P4 (D-738) | Pass 4 | 9 | 4H + 3M + 1L + 1OBS-deferred |
| FB-IMPL-P5 (D-739) | Pass 5 | 5 | 1H + 2M + 2L + 1OBS-deferred |
| **TOTAL** | | **50** | **10H + 11M + 9L + 2C + 12OBS (2 deferred)** |

Note: Pass-3 findings were classified CRITICAL by adversary (per pass-3 report); recorded as C above for audit fidelity.

## Scope Changes Summary

**Architect (2 dispatches):**
- ADR-028 v1.0 → v1.1: §D2 Armis row corrected (`api_key` → `bearer_static` per `prism-dtu-armis/src/lib.rs:16-17` BearerStatic enforcement); ARCH-INDEX v2.86 → v2.87
- ADR-028 v1.1 → v1.2: §D2 cyberint cite symbol-anchored to `::extract_session_token()` per TD-VSDD-091; ARCH-INDEX v2.87 → v2.88

**Product-Owner:**
- BC-2.16.001 frontmatter `modified: "2026-05-20"` (F-LP5-MED-002 POL-27 fix)
- BC-2.16.009 frontmatter `modified: "2026-05-20"` (POL-29 sibling sweep — bonus closure beyond adversary report scope)
- BC-2.16.013 v1.4 → v1.5: cyberint `extract_session_token` symbol anchor per TD-VSDD-091 + POL-25 multi-cite sweep
- HS-015 v1.1 → v1.2: 2 cyberint citation sites updated to symbol anchor
- BC-INDEX v5.25 → v5.26
- HOLDOUT-INDEX v1.5 → v1.6

**Story-Writer:**
- PLUGIN-MIGRATION-001-D story v1.4 → v1.5: `crates_touched` frontmatter add `prism-core` (F-LP5-MED-001 POL-13 fix); Task 11 `#[non_exhaustive]` phrasing unconditional + investigation result (SpecErrorCode NOT in non-exhaustive-violation gate scope, EXPECTED stays 32); cyberint symbol anchor propagated to AC-011 + Task 5; BC-2.16.013 pin sweep v1.4 → v1.5 across 5 active-prose sites per POL-23
- STORY-INDEX v2.161 → v2.162

**State-Manager (this burst):**
- local-pass-5.md persisted (adversary lacked write access at runtime)
- fix-burst-5.md (this file)
- input-hash updated: story v1.5 `input-hash: "1b815e4"` (SHA256 of story file content, first 7 chars)
- STATE.md v7.425 → v7.426

## Streak Status

- streak_before_pass5: 0/3
- streak_after_fb5: 0/3 (unchanged — this is a fix-burst, not an adversary pass)
- next_action: Pass-6 fresh-context adversary dispatch

## Next

Pass-6 with fresh-context adversary. Target streak 0/3 → 1/3 per BC-5.39.001 / D-716 Option A standing.
