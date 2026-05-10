---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T23:59:00Z
phase: 5
pass: 13
traces_to: ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
review_id: ADR-023-pass-13
date: 2026-05-10
reviewer: adversary
target_artifact_sha_at_review: "bc8ed323"
target_artifact_version: "v1.9"
findings_total: 1
findings_by_tier:
  CRIT: 0
  HIGH: 1
  MED: 0
  LOW: 0
  OBS: 0
process_gap_findings: 0
pass_number: 13
previous_review: "ADR-023-pass-12.md"
convergence_status: NOT_CLEAN
fix_burst_required: true
residuals_from_previous_pass: 0
new_findings_this_pass: 1
streak_status: "0/3 RESET (pass-13 NOT_CLEAN — 1 HIGH; streak was 1/3 after pass-12 CLEAN)"
trajectory: "26→16→12→14→3→3→1→0→0→4→2→0→1"
verifications_performed: 22
related_tasks: []
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/cycles/wave-4-operations/adversarial-reviews/ADR-023-pass-12.md"
  - "crates/prism-bin/src/boot.rs"
input-hash: "[live-state]"
---

# Adversarial Review: ADR-023 Plugin-Only Sensor Architecture (Pass 13)

## Finding ID Convention

Finding IDs use the pass-13-scoped format:

- `F-PASS13-{CRIT,HIGH,MED,LOW,OBS}-NNN` — finding in pass-13

This pass surfaces **ONE HIGH finding**: F-PASS13-HIGH-001 — sibling-site propagation gap
in the TD-PLUGIN-SIGNING-001 target release designation. Pass-10's fix-burst-8 introduced
the VP-PLUGIN-007 lifecycle note at L737-744 which correctly uses "v1.0+N" at L741 but then
immediately contradicts itself at L743 by reverting to "v1.0+1". Two companion sites at L848
and L851 (Consequences/Negative section) carry the same stale "v1.0+1" designation. The
S-7.01 sibling-site propagation pattern continues to recur across fix-burst passes.

The streak resets from 1/3 to 0/3. Fix-burst-10 dispatches in the same burst to close this
finding at all 3 sites as ADR-023 v1.10.

---

## Summary — 1 HIGH FINDING. NOT_CLEAN. STREAK RESET 1/3 → 0/3.

Pass-13 fresh-context review of ADR-023 v1.9 (SHA `bc8ed323`) yields **ONE finding**:
1 HIGH, 0 CRIT, 0 MED, 0 LOW, 0 OBS. The 3-CLEAN convergence streak resets from 1/3 to 0/3.

Twenty-two source-of-truth verifications were executed. Twenty-one PASS, one FAIL (the
finding sites).

The one finding from pass-12 is confirmed closed (F-PASS11-HIGH-001 scoping at all 8 sites
still holds; F-PASS11-LOW-001 duplicate still deleted). Zero residuals from pass-12.

The new finding is novel: pass-10's VP-PLUGIN-007 lifecycle note amendment introduced an
internal contradiction. L741 correctly says "v1.0+N when first non-trivial third-party WASM
plugin is genuinely needed" — this is the canonical phrasing adopted in v1.3 fix-burst-4
(F-PASS4-HIGH-001 closure). But L743 immediately says "TD-PLUGIN-SIGNING-001 target release
is v1.0+1" — reverting to the pre-v1.3 phrasing that v1.4's F-PASS4-HIGH-001 closure was
specifically designed to eliminate. Two companion sites at L848 and L851 in the
Consequences/Negative section carry identical "v1.0+1" phrasing.

The trajectory is now:
**26→16→12→14→3→3→1→0→0→4→2→0→1 (streak RESET; S-7.01 sibling-site propagation
pattern recurs in pass-10 amendment; fix-burst-10 dispatches same burst)**

Pass-14 is the next dispatch. Target: streak 1/3.

---

## Part A — Fix Verification

### Pass-12 Closure Verification

Pass-12 raised ZERO findings. This review verifies the v1.9 state continues to hold at all
previously-corrected sites before reporting new findings.

**F-PASS11-HIGH-001 (8 sites)** — The F-PASS10-HIGH-001 scoping applied to 5 sites (L275,
L732, L809-810, L841, L986) plus 3 additional reader-visible sites (L142-146 Decision
opening, L264-265 Rule 4 body, L789-795 Consequences/Positive).

Verification result: CLOSED at all 8 sites. All sites continue to carry qualified language
distinguishing third-party/external plugins from bundled first-party platform plugins. No
regression.

**F-PASS11-LOW-001** — Verbatim duplication at L120-122 vs L123-125.

Verification result: CLOSED. Single instance of boot.rs removal prose remains at L123-125.
No duplication.

Summary: zero pass-12 residuals.

---

## Part B — New Findings

### CRITICAL

_None._

---

### HIGH

**F-PASS13-HIGH-001** — TD-PLUGIN-SIGNING-001 target release: "v1.0+1" vs "v1.0+N" internal
contradiction introduced by pass-10 amendment at VP-PLUGIN-007 lifecycle note and
Consequences/Negative section.

**Classification:** HIGH — Reader-visible internal contradiction within ADR-023 v1.9 itself
(not a cross-document drift). The document simultaneously presents two incompatible
specifications of the same TD target. A reader of the VP-PLUGIN-007 lifecycle note encounters
L741 saying "v1.0+N when first non-trivial third-party WASM plugin is genuinely needed"
followed TWO LINES LATER by L743 saying "v1.0+1 (signing infrastructure deferred even though
first-party OCSF complex-transform plugins exist in v1.0)". This is not an ambiguity — it is
a direct contradiction within a 7-line note. The Consequences/Negative section at L848+L851
repeats the contradiction.

**Background:** v1.3 fix-burst (F-PASS4-HIGH-001 closure) deliberately replaced "v1.0+1" with
"v1.0+N when first non-trivial third-party WASM plugin is genuinely needed" at 3 sites (C4,
VP-PLUGIN-004 twice). The rationale was explicit: the signing release target depends on when
a third-party plugin is genuinely needed, not a fixed release number. v1.4 changelog confirms:
"F-PASS4-HIGH-001 closed: 'v1.0+1' replaced with 'v1.0+N when first non-trivial third-party
WASM plugin is genuinely needed' at 3 sites". The canonical phrasing is therefore "v1.0+N
when first non-trivial third-party WASM plugin is genuinely needed".

**Pass-10's introduction of the contradiction:** Fix-burst-8's VP-PLUGIN-007 lifecycle note
amendment (L737-744) was written as a new block. L741 correctly used the v1.0+N canonical
phrasing. L743 inadvertently reverted to "v1.0+1" — the pre-v1.3 stale phrasing. This
matches the S-7.01 partial-fix propagation pattern: a new amendment block introducing a
sibling-site inconsistency not caught by the same-burst verification.

**Affected sites (8 file:line citations):**

1. `ADR-023.md:L741` — "VP-PLUGIN-007 becomes a v1.0+N candidate for amendment when the
   first non-trivial third-party WASM plugin is genuinely needed." **CANONICAL — CORRECT**
2. `ADR-023.md:L742` — "TD-PLUGIN-SIGNING-001 target release is v1.0+1 (signing
   infrastructure deferred even though first-party OCSF complex-transform plugins exist in
   v1.0)." **STALE — NEEDS FIX**
3. `ADR-023.md:L743` — (same line continuation as L742 in some reads) — the closing
   parenthetical of the L743 sentence carries "v1.0+1". **STALE — NEEDS FIX**
4. `ADR-023.md:L848` — "v1.0 ships first-party in-repo OCSF complex-transform plugins
   UNSIGNED with explicit security warning + audit log per TD-PLUGIN-SIGNING-001 P0 v1.0+1
   target." **STALE — NEEDS FIX**
5. `ADR-023.md:L851` — "TD-PLUGIN-SIGNING-001 target release is v1.0+1 (signing
   infrastructure deferred even though first-party OCSF complex-transform plugins ship in
   v1.0)." **STALE — NEEDS FIX**
6. `ADR-023.md:L564` — "Plugin signing is deferred to v1.0+N when first non-trivial
   third-party WASM plugin is genuinely needed per TD-PLUGIN-SIGNING-001." **CANONICAL — CORRECT**
7. `ADR-023.md:L683` — "signing is deferred to v1.0+N when first non-trivial third-party
   WASM plugin is genuinely needed per TD-PLUGIN-SIGNING-001" **CANONICAL — CORRECT**
8. `ADR-023.md:L687` — "For v1.0+N: property will be amended to assert that unsigned plugins
   fail to load with a structured error." **CANONICAL — CORRECT**

Summary of 8 citations: 5 sites carry correct "v1.0+N when..." phrasing (L564, L683, L687,
L741, and one additional canonical site at L991 in §F Amendment Status). 3 sites carry stale
"v1.0+1" phrasing (L743, L848, L851). Fix: replace "v1.0+1" with "v1.0+N when first
non-trivial third-party WASM plugin is genuinely needed" at the 3 stale sites. Rewrite the
closing parenthetical at L743+L851 to match the structure of the canonical L741 phrasing
(no parenthetical needed — the reason IS the canonical phrase).

**Fix scope:** Edit-only per TD-FACTORY-HOOK-BYPASS-001 P1. No architectural change — the
canonical phrasing is already established; this is a wording consistency fix. Fix-burst-10
dispatches in the same burst.

---

### MEDIUM

_None._

---

### LOW

_None._

---

### OBS (Out-of-Scope Observations — Not Findings)

_None._

---

## Verifications Performed (22 checks)

All 22 source-of-truth verifications executed against ADR-023 v1.9 (SHA `bc8ed323`) and
sibling document `crates/prism-bin/src/boot.rs`.

| # | Check | Target | Result |
|---|-------|--------|--------|
| SOT-01 | Story count consistency | `13 stories` cited at frontmatter + summary + Wave table + Wave 0 section + PREREQ-F section | PASS — 13 consistent throughout |
| SOT-02 | Story point arithmetic (Wave 1) | Wave 1: 95 SP total claimed | PASS — D+E+A+B+C row sums to 95; subtotals correct |
| SOT-03 | Story point arithmetic (Wave 2) | Wave 2: 146 SP total claimed | PASS — arithmetic consistent across all Wave 2 rows |
| SOT-04 | VP-PLUGIN registration | VP-PLUGIN-001..006 cited in ADR-023 body | PASS — VP-INDEX registers VP-146..VP-152 as aliases; all present |
| SOT-05 | BC frontmatter `scheduled_amendment_in` | Wave 0/F prerequisite BCs cited | PASS — `scheduled_amendment_in: wave-0-prereq-f` present in referenced BCs |
| SOT-06 | DI-012 annotation | DI-012 back-reference cited | PASS — DI-012 annotated in domain-spec/invariants.md with cross-reference to ADR-023 §B.2 |
| SOT-07 | Input-hash real | `input-hash:` field not a bracketed placeholder | PASS — input-hash contains real value `2f64319`, not `[placeholder]` |
| SOT-08 | Process-Gap Awareness section | ADR-023 §G Process-Gap Awareness exists | PASS — section present at expected location |
| SOT-09 | Edit-only discipline | No ADR-023 content rewritten wholesale | PASS — changelog shows incremental fix-burst entries; no wholesale rewrite detected |
| SOT-10 | Version stamp body-wide consistency | `version: "v1.9"` in frontmatter; body Status block at L80 and L856 | PASS — Status block cites v1.9 at both locations; changelog shows v1.9 row; consistent |
| SOT-11 | Wave 0/F PREREQ-F dependency chain | S-PLUGIN-PREREQ-F blocks PREREQ-A through PREREQ-E | PASS — dependency arrows correct in Wave 0 table |
| SOT-12 | TD-VERSION-STAMP-SWEEP-001 reference | Process-Gap section cites TD-VERSION-STAMP-SWEEP-001 | PASS — TD registered and cited at §G |
| SOT-13 | Changelog immutability | Prior changelog rows (v1.1..v1.8) unchanged | PASS — rows match prior observed text verbatim; immutable audit trail intact |
| SOT-14 | F-PASS11-HIGH-001 closure: Decision opening scoped | L142-146 Decision opening carries qualified "zero third-party/external" language | PASS — scoped language present; unqualified claim removed |
| SOT-15 | F-PASS11-HIGH-001 closure: Rule 4 body scoped | L264-265 Rule 4 "zero plugins at launch" baseline carries qualifying scope | PASS — scoped language present; unqualified baseline removed |
| SOT-16 | F-PASS11-HIGH-001 closure: Consequences/Positive scoped | L789-795 Consequences/Positive section scoped to third-party/external | PASS — scoped language present; contradiction with Wave 1/C in-repo OCSF plugin resolved |
| SOT-17 | Corpus-wide grep: F-PASS11 sites still closed | Full document sweep for pre-fix "zero in-repo plugins" phrasing | PASS — zero additional sites found; 8-site scoping remains comprehensive |
| SOT-18 | F-PASS11-LOW-001 closure: duplicate deleted | Context section L120-122 duplicate paragraph removed | PASS — single instance of boot.rs removal prose at L123-125; no duplication |
| SOT-19 | v1.0+N canonical sites (5) | L564 + L683 + L687 + L741 + L991 carry "v1.0+N when first non-trivial third-party WASM plugin is genuinely needed" | PASS — 5 canonical sites confirmed correct |
| SOT-20 | v1.0+1 stale sites (3) | L743 + L848 + L851 carry stale "v1.0+1" phrasing | FAIL — 3 stale sites confirmed; F-PASS13-HIGH-001 |
| SOT-21 | v1.8 fix sites still hold (L275, L732, L809-810, L841, L986) | Five sites corrected in v1.8 remain correctly scoped in v1.9 | PASS — no regression; v1.8 scoping preserved through v1.9 edits |
| SOT-22 | Rule 4 ↔ Rule 5 coherence (post-v1.9) | Rule 4 and Rule 5 logically consistent after scoping corrections | PASS — Rule 4 extension mechanism + Rule 5 no-built-in-sensors coherent; no new contradictions |

**PASS: 21 / FAIL: 1**

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 1 |
| MEDIUM | 0 |
| LOW | 0 |
| OBS | 0 |

**Overall Assessment:** NOT_CLEAN
**Convergence:** STREAK RESET 1/3 → 0/3 (pass-13 NOT_CLEAN)
**Readiness:** fix-burst-10 dispatches in same burst; pass-14 to follow

---

## Convergence Assessment

Pass-13 is NOT_CLEAN. The 3-CLEAN convergence streak resets from 1/3 to 0/3.

F-PASS13-HIGH-001 surfaces a novel internal contradiction introduced by fix-burst-8's
VP-PLUGIN-007 lifecycle note. The contradiction is at the sentence level within a 7-line
block: L741 says "v1.0+N when first non-trivial third-party WASM plugin is genuinely needed"
(canonical) and L743 says "v1.0+1" (stale). Two companion sites at L848 and L851 repeat the
stale designation.

The S-7.01 partial-fix propagation pattern continues to surface across amendment passes. The
pattern has now manifested at:
- Pass-4 → fix-burst-4: v1.0+1 → v1.0+N at 3 sites (C4, VP-PLUGIN-004 twice)
- Pass-5: F-PASS5-HIGH-001 status block version-stamp sibling site
- Pass-6: F-PASS6-HIGH-001 VP-PLUGIN-006 Phase: migration sibling site
- Pass-7: F-PASS7-HIGH-001 body status block L80+L850 version stamp
- Pass-10 → fix-burst-8: scoping at 5 sites, missed 3 reader-visible sites
- Pass-11: F-PASS11-HIGH-001 propagation gap at 3 sites
- Pass-13: F-PASS13-HIGH-001 v1.0+1 re-introduced at 3 sites by fix-burst-8's new
  VP-PLUGIN-007 lifecycle note block

Fix-burst-10 dispatches in the same burst: replace "v1.0+1" with "v1.0+N when first
non-trivial third-party WASM plugin is genuinely needed" at L743, L848, and L851. Body
version sweep v1.9→v1.10. Bump frontmatter version. Single atomic burst + commit per
TD-FACTORY-HOOK-BYPASS-001 and Single-Commit Burst Protocol (TD-VSDD-053).

Pass-14 should be dispatched as a fresh-context adversarial review of v1.10 at HEAD frozen
at post-fix-burst-10 SHA. Target: streak 1/3.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 13 |
| **New findings** | 1 |
| **Duplicate/variant findings** | 0 (S-7.01 pattern class recurs but finding is at novel sites) |
| **Novelty score** | HIGH — finding is structurally novel (internal contradiction within a 7-line block introduced by the same pass that corrected 5 other sites) |
| **Median severity** | HIGH |
| **Trajectory** | 26→16→12→14→3→3→1→0→0→4→2→0→1 |
| **Verdict** | FINDINGS_REMAIN — streak reset 1/3 → 0/3; fix-burst-10 dispatches same burst; pass-14 target streak 1/3 |

The S-7.01 sibling-site propagation pattern has now recurred 7 times across the ADR-023
adversarial review series. Each recurrence surfaces at a different location in the document.
The pattern's persistence suggests that the TD-VERSION-STAMP-SWEEP-001 and corpus-wide grep
disciplines are necessary but not sufficient: they catch missing versions and scoping gaps but
not contradictions introduced within the same new amendment block. Pass-14 should include an
explicit intra-block consistency check: for each amended block, verify all sentences within
the block are mutually consistent on version references.
