---
document_type: fix-burst-closure-report
target_artifact: S-PLUGIN-PREREQ-D
fix_burst_N: 23
target_pass: 25
findings_closed: 3 (1 HIGH F-LP25-HIGH-001 spawn_blocking re-anchor; 2 LOW F-LP25-LOW-001 SS-17 short-name + F-LP25-LOW-002 AC-9 plugin HTTP defaults hedge)
findings_closed_burst_a: 1 MEDIUM (F-LP25-MED-001 STORY-INDEX date drift — closed in Burst A D-513)
findings_deferred: 1 OBS (F-LP25-OBS-001 BC-2.17.002 EC-17-007 vacuous-truth under Vec<String> — routed to product-owner phase-5 in Burst A D-513)
producer: state-manager (orchestrator-coordinated; story-writer Stage 1 + state-manager Stage 2 — single commit per TD-VSDD-053)
story_v_before: 1.22
story_v_after: 1.23
factory_shas: [c4f42b0b, e8d5bd0a, "TBD (see STATE.md D-514 row for authoritative this-burst SHA)"]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → CLOSED"
next_action: "Adversary pass-26 dispatch — target streak 0/3 → 1/3 if CLEAN; rigorous re-verification of spawn_blocking anchor (new citation BC-2.17.005 §Invariants must be independently verified in fresh-context)"
codification_candidate_11: "lexical-vs-semantic sweep recurrence (6th instance per v1.19 changelog) — adversary regexes that find a string in story body without verifying the cited ADR/BC actually contains it. Process-gap tagged HIGH at F-LP25-HIGH-001."
---

# S-PLUGIN-PREREQ-D Fix-Burst-23 Closure Report

**Fix-burst-23 CLOSED: 3/3 in-scope findings (1H+2L); 1 OBS deferred to phase-5 (prior Burst A D-513)**
**Dispatch: story-writer (Stage 1 @ story v1.22 → v1.23) + state-manager (Stage 2 — this commit)**
**19th consecutive single-commit-with-TBD-pin (TD-VSDD-053; F-LP10-OBS-001 DECISIVELY STABLE)**

---

## Closures Table

| Finding | Severity | Closed By | Stage | Method |
|---------|----------|-----------|-------|--------|
| F-LP25-HIGH-001 | HIGH | story-writer | 1 | Architecture Compliance Rules row re-anchored from fabricated `ADR-023 §C4` → canonical `BC-2.17.005 §Invariants`; grep verification: `spawn_blocking` absent from ADR-023; present at BC-2.17.005 lines 51, 73 |
| F-LP25-LOW-001 | LOW | story-writer | 1 | SS-17 YAML comment normalized: "Plugin Runtime" → "WASM Plugin Runtime" (per ARCH-INDEX v2.43 canonical name; POL-6) |
| F-LP25-LOW-002 | LOW | story-writer | 1 | AC-9 trace header stripped fabricated prose "ADR-023 §C4 plugin HTTP defaults +"; canonical `BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005` retained; no structural change to AC-9 body |

## Prior Burst A Closure (D-513 — already recorded)

| Finding | Severity | Closed By | Burst | Method |
|---------|----------|-----------|-------|--------|
| F-LP25-MED-001 | MEDIUM | state-manager | A (D-513) | STORY-INDEX date drift: PREREQ-D row date 2026-05-14 → 2026-05-13 (story frontmatter `timestamp: "2026-05-13T10:30:00Z"` is source of truth per POL-13). STORY-INDEX v2.90→v2.91. |

## Deferred Findings (Phase-5 carry-forward — prior Burst A D-513)

| Finding | Severity | Routing | Rationale |
|---------|----------|---------|-----------|
| F-LP25-OBS-001 | OBS | phase-5 product-owner adjudication | BC-2.17.002 EC-17-007 vacuous-truth under `Vec<String>` — empty-allowlist branch semantics question; requires PO adjudication on whether EC-17-007 is a well-formed error condition under the `Vec<String>` type; out-of-perimeter for story-writer; 5th deferred-findings entry in `cycles/wave-4-operations/deferred-findings-phase-5.md`. |
| F-LP16-OBS-001 | OBS | phase-5 architect adjudication | prism-bin/Cargo.toml edition 2021 vs canonical 2024; workspace-wide edition unification |
| F-LP19-LOW-002 | LOW | phase-5 PO/architect adjudication | VP-INDEX VP-PLUGIN-004 framing vs BC-2.16.002 v1.12 catalog discipline |
| F-LP22-OBS-001 | OBS | phase-5 architect adjudication | `PluginError` lacks `#[non_exhaustive]` (prism-core scope; compile-fail gate EXPECTED=30 impact) |

---

## Story-Writer Stage 1 Detail

**Factory SHAs (prior commits in cascade):** c4f42b0b (pass-25 report commit), e8d5bd0a (D-513 Burst A)
**Story transition:** v1.22 → v1.23

### F-LP25-HIGH-001 Closure — spawn_blocking Anchor Re-anchor

**Root cause:** Architecture Compliance Rules table row cited `ADR-023 §C4` as the authority for the `spawn_blocking` mandate. Pass-25 idempotency audit independently opened ADR-023 and grepped for `spawn_blocking` — the term does not appear anywhere in ADR-023 §C4 or any other section of ADR-023. The canonical location is BC-2.17.005 §Invariants (verified at lines 51 and 73).

| Site | Before | After |
|------|--------|-------|
| Architecture Compliance Rules row (line ~980) | `\| Plugin compilation MUST run in \`spawn_blocking\` \| ADR-023 §C4 \| Code review; tokio lint \|` | `\| Plugin compilation MUST run in \`spawn_blocking\` \| BC-2.17.005 §Invariants \| Code review; tokio lint \|` |

**Process-gap significance:** This is the 6th instance of the lexical-vs-semantic sweep pattern (codification candidate #11). Prior passes 1–24 confirmed the string "ADR-023 §C4" was present in the story body but did NOT open ADR-023 and grep for whether §C4 content actually contained the `spawn_blocking` mandate. Pass-25 idempotency broke this pattern by independently verifying the cited document content. POL-22 Phase A must be extended to require opening the cited target document, not merely confirming the citation text string exists in the story body.

**Sibling-site sweep result:** Zero additional sites in the active story body cite `ADR-023 §C4` for the `spawn_blocking` mandate. The changelog history entries retain the old phrase as immutable audit trail (POL-1 exempt).

### F-LP25-LOW-001 Closure — SS-17 YAML Comment Short-Name

**Root cause:** YAML frontmatter comment read `#   SS-17 (Plugin Runtime, prism-spec-engine)` — the parenthetical short-name "Plugin Runtime" is incomplete. ARCH-INDEX v2.43 canonically registers SS-17 as "WASM Plugin Runtime" (full name distinguishing it from bare plugin infrastructure).

| Site | Before | After |
|------|--------|-------|
| SS-17 YAML comment (line ~47) | `#   SS-17 (Plugin Runtime, prism-spec-engine) owns all sandbox BCs...` | `#   SS-17 (WASM Plugin Runtime, prism-spec-engine) owns all sandbox BCs...` |

**Sibling-site sweep result:** No additional sites in the active story body use the truncated "Plugin Runtime" name for SS-17. The full name "WASM Plugin Runtime" is used consistently elsewhere in the story body.

### F-LP25-LOW-002 Closure — AC-9 Trace Header Hedge Removal

**Root cause:** AC-9 trace header contained the prose fragment "traces to ADR-023 §C4 plugin HTTP defaults + BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005". The "ADR-023 §C4 plugin HTTP defaults +" portion is fabricated — ADR-023 §C4 does not contain plugin HTTP defaults language, and the anchor is redundant to BC-2.17.002 which is the canonical error-condition authority. The hedge was removed; the canonical citation to BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005 is retained.

| Site | Before | After |
|------|--------|-------|
| AC-9 trace header (line ~367) | `### AC-9 — ... (traces to ADR-023 §C4 plugin HTTP defaults + BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005; closes TD-S-PLUGIN-PREREQ-B-005)` | `### AC-9 — ... (traces to BC-2.17.002 v1.5 §Error Conditions E-PLUGIN-005; closes TD-S-PLUGIN-PREREQ-B-005)` |

**Sibling-site sweep result:** All remaining AC-9 references in the active body cite only the canonical BC-2.17.002 anchor. Zero active-body sites retain the fabricated "ADR-023 §C4 plugin HTTP defaults" prose.

---

## Frontmatter Update

| Field | Before | After |
|-------|--------|-------|
| `version` | `"1.22"` | `"1.23"` |
| `timestamp` | `"2026-05-13T10:30:00Z"` | `"2026-05-13T14:00:00Z"` |
| Changelog | — | v1.23 row inserted above v1.22 row |

---

## Sibling-Site Sweep Summary (3/3 axes CLEAN)

| Axis | Query | Result |
|------|-------|--------|
| `spawn_blocking` anchor | grep `ADR-023 §C4` in active story body (excluding changelog) | ZERO hits — fabricated anchor fully removed |
| SS-17 short-name | grep `(Plugin Runtime,` in active story body | ZERO hits — normalized to "WASM Plugin Runtime" |
| AC-9 fabricated hedge | grep `ADR-023 §C4 plugin HTTP defaults` in active story body | ZERO hits — stripped from trace header |

All three sibling-site sweeps confirm the active body is clean. Changelog history rows retain the old phrases as immutable audit trail per POL-1.

---

## Why Pass-25 Caught What 24 Prior Passes Missed

**Process-gap insight (codification candidate #11 — 6th recurrence):**

Passes 1–24 applied POL-22 Phase A external-anchor verification by confirming that the *citation text* appeared in the story body. For example, they would confirm that the string "ADR-023 §C4" was present in the Architecture Compliance Rules row — and stop there. This is a **lexical** match, not a **semantic** verification.

Pass-25 was an idempotency check dispatched at an unchanged HEAD (story v1.22 SHA a9a51671). The idempotency methodology required the adversary to independently verify that the cited document (ADR-023) actually contained the content implied by the citation (§C4 containing the `spawn_blocking` mandate). When the adversary opened ADR-023 and grepped for `spawn_blocking`, it was absent. The citation text matched syntactically but was semantically wrong — ADR-023 §C4 is about a different constraint, and the actual `spawn_blocking` mandate lives in BC-2.17.005 §Invariants.

**Root pattern:** Prior passes performed **syntactic citation matching** ("does the story cite ADR-023 §C4?"). They did NOT perform **semantic citation validation** ("does ADR-023 §C4 actually contain the rule cited?"). This is a systematic gap in POL-22 Phase A as previously implemented. It is the same lexical-vs-semantic anti-pattern that produced F-LP13/F-LP14/F-LP18/F-LP19/F-LP20-LOW instances earlier in the cascade (5 prior recurrences).

**Candidate #11 process-improvement:** POL-22 Phase A MUST require opening the cited target document and grepping for the claim's key term before declaring an anchor PASS. Story-body substring match is NOT sufficient.

---

## Process-Gap Codifications (11 active — candidate #11 new at pass-25)

| # | Candidate Name | Threshold | Status | Evidence |
|---|---------------|-----------|--------|---------|
| 1 | `version-pin-sweep-on-every-fix` | 3-instance | ACTIVE | F-LP7/F-LP9/F-LP20 |
| 2 | `sibling-prose-sweep-all-18-sections` | 3-instance | ACTIVE | F-LP13/F-LP14/F-LP19 |
| 3 | `version-pin-drift-sub-pattern` | 3-instance | ACTIVE | F-LP18/F-LP19/F-LP20 |
| 4 | `story-writer-template-enforcement-for-risk-HIGH` | 1-instance HIGH-sev | ACTIVE | F-LP17-OBS-001 |
| 5 | `lexical-vs-semantic-sweep` | 5-instance (now 6) | ACTIVE | F-LP13/F-LP14/F-LP18/F-LP19/F-LP19-OBS/F-LP25-HIGH-001 |
| 6 | `adversary-must-verify-own-fix-prescriptions` | 1-instance HIGH-sev | ACTIVE | F-LP16-HIGH-001 |
| 7 | `state-manager-attempts-unauthorized-push` | 1-instance P0 | ACTIVE | Post-fix-burst-15 security incident |
| 8 | `adversary-must-verify-external-anchors-recursively-on-every-pass` (POL-22 Phase A) | 3-instance | ACTIVE | F-LP15/F-LP16/F-LP21 |
| 9 | `test-crate-sites-must-be-enumerated-alongside-production-sites` | 1-instance | MONITORING | F-LP22-MED-001 |
| 10 | `internal-cross-reference-type-unification-verification` (POL-22 Phase B candidate) | 4-instance | ACTIVE | F-LP23-HIGH-001 (4th regression: pass-7 paths; pass-15→16 PrismError variant; pass-21 PipelineError; pass-23 Option<Vec>) |
| **11** | **`lexical-vs-semantic-anchor-content-verification`** (POL-22 Phase A extension) | **6-instance** | **ACTIVE** | **F-LP25-HIGH-001 (6th: F-LP13+F-LP14+F-LP18+F-LP19+F-LP20 prior; pass-25 idempotency caught syntactic-match-without-semantic-open-and-grep gap)** |

---

## Convergence Status

- **Pass-25:** BLOCKED (4 findings: 1H+1M+2L+1OBS) — idempotency caught anchor drift missed by 24 passes
- **Fix-burst-23:** CLOSED — 3/3 in-scope (1H+2L); 1M closed in Burst A; 1 OBS deferred to phase-5
- **Streak:** 0/3 HOLD — fix-burst-23 does not advance streak; pass-26 next
- **Trajectory:** `16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → CLOSED`
- **Next action:** Adversary pass-26 dispatch at story v1.23. POL-22 Phase A reinforced: adversary must open and grep cited target documents (BC-2.17.005 for `spawn_blocking`; ADR-023 for absence of `spawn_blocking`/`plugin HTTP defaults`). Three-CLEAN window (0/3 → 1/3 → 2/3 → 3/3 per BC-5.39.001) opens at pass-26 CLEAN.

**Special verification required at pass-26:**
- MUST grep BC-2.17.005 for `spawn_blocking` to confirm F-LP25-HIGH-001 fix holds (re-anchor to BC-2.17.005 §Invariants is correct)
- MUST confirm `ADR-023 §C4` is absent from active story body (Architecture Compliance Rules row + AC-9 + all other sites)
- MUST confirm `(Plugin Runtime,` is absent from active story body (SS-17 YAML comment)
