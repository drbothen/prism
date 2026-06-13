---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr_number: 185
pass_number: 8
cascade: PR-LEVEL (distinct from LOCAL; LOCAL CONVERGED 3/3 strict @13 passes)
base_develop: "939f36ce"
feature_head_at_review: "bc0f36c5"
feature_head_after_fix_burst: "bc0f36c5"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-12
authority: BC-5.39.001 D-779
decision: D-1114
---

# PR-LEVEL Adversary Pass 8 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B — Scenario Progression + Enrichment Correlation Live Demo
**PR:** #185 (base develop@939f36ce, head bc0f36c5 — unchanged from pass 7; no code change since pass 4)
**Pass:** PR-LEVEL pass 8 (distinct from LOCAL cascade; LOCAL CONVERGED 3/3 strict at 13 passes)
**Date:** 2026-06-12

## Pass-7 Closure Verification

All pass-7 findings verified sound:

- **BPRL-P7-01 MED [process-gap]** (BC-2.06.019 v1.6 inventory note fabricated grep claim for claroty/alerts.rs): CLOSED. BC-2.06.019 v1.6→v1.7 — single sentence corrected: claroty/alerts.rs does NOT appear in either grep set; zero stage/mask references; EXEMPT stands solely on real-API grounds; fabricated justification removed. Route Coverage Table unchanged (8 rows, EXHAUSTIVE); no semantic/contract change. Story B v2.9. PIVOT-003 v1.4. STORY-INDEX v2.362. BC-INDEX v6.35. Story B HEAD bc0f36c5 UNCHANGED. **VERIFIED — BC-2.06.019 v1.7 inventory note correctly states claroty/alerts.rs does NOT appear in grep set; EXEMPT on real-API grounds. CLOSED stands.**

## Pass-8 Finding

### BPRL-P8-01 — MED [process-gap] BC-INDEX row 120 story-version pin stale (v2.4) after D-1113 story B v2.9 advance

**Severity:** MED
**Category:** process-gap (state-manager bookkeeping gap — index-row annotation sync is state-manager domain)
**Location:** `.factory/specs/behavioral-contracts/BC-INDEX.md` line 120 (BC-2.06.020 row)

**Finding:**

BC-INDEX row 120 (BC-2.06.020 — Demo-Server Enrichment Correlation) contains the annotation:

> `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.4 (B-P5-03 2026-06-12)`

The current story B version is v2.9 (confirmed via `.factory/stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md` frontmatter `version: "2.9"`).

The D-1113 burst that advanced story B from v2.8→v2.9 updated row 119 (BC-2.06.019) to `ready v2.9 (D-1113 2026-06-12)` correctly, but did NOT sweep row 120 (BC-2.06.020) — which was last updated at v2.4 by the B-P5-03 micro-burst.

**Evidence:**
- `grep -n "S-DEMO-DTU-LIVE-SCENARIO-001-B ready v" .factory/specs/behavioral-contracts/BC-INDEX.md` (table rows only):
  - Row 119: `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9 (D-1113 2026-06-12)` — CURRENT
  - Row 120: `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.4 (B-P5-03 2026-06-12)` — STALE

**Root cause:**

D-1113 burst description included "row 119 (BC-2.06.019) correctly says `ready v2.9` — the finding is that row 120 was not swept." The prior version-pin sync bursts (B-P5-03 at v6.31, which synced both rows 119 and 120 to v2.4) established the precedent that BOTH rows must be swept together when story B advances. D-1113 updated only the row that was the subject of the BC amendment (row 119 for BC-2.06.019) without applying the exhaustive same-class sweep to sibling row 120 (BC-2.06.020 — also anchored to story B).

**All other axes verified clean (code unchanged since pass 4):**

- **BC-2.06.019 v1.7 Route Coverage Table:** PASS — 8 rows, EXHAUSTIVE; inventory note corrected in v1.7 (claroty/alerts.rs EXEMPT on real-API grounds).
- **BC-2.06.020 invariants:** PASS — enrichment correlation BC content unchanged and consistent.
- **E-DEMO-006 byte-exact:** PASS — org_id guard message format matches error-taxonomy v1.78 verbatim.
- **SAP-1:** PASS — `rg 'event_type\s*=' crates/ --type rust` — no new `event_type` emissions in PR diff.
- **SAP-2:** N/A — no sensor TOML files in PR diff.
- **Forbidden-pattern sweep:** PASS — no `unwrap()`, no `println!`, no `reqwest::Client::new()` without timeout, no retired ColumnType variants.
- **DormantTenant regression guard:** PASS — Red Gate test 17 present and non-vacuous.
- **Demo evidence 18/18 ACs:** PASS — demo evidence in commit range intact.
- **Frontmatter-body coherence:** PASS — acceptance_criteria_count 18, red_gate_tests 19 consistent.
- **Story B HEAD:** bc0f36c5 = remote (no code change since pass 4).
- **VP-INDEX and ARCH-INDEX:** Both reference story B by story ID only, no version pin annotations — no stale pins.
- **PIVOT-001/002/003 BC-INDEX rows:** No version pin annotations in table rows for these stories — no stale pins.

**CLEAN(strict):** no (1 MED process-gap finding)
**CLEAN(PR-merge):** no (1 MED finding)
**Streak:** 0/3

---

## Closure Evidence (same-session fix burst D-1114)

**State-manager amended BC-INDEX row 120:**

- **Before (v6.35):** `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.4 (B-P5-03 2026-06-12)`
- **After (v6.36):** `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9 (D-1114 2026-06-12)`

**Exhaustive annotation sweep (shared-anchor-story index rows as a class):**

Sweep command: `grep -n "S-DEMO-DTU-LIVE-SCENARIO-001-B ready v\|PIVOT-001 ready v\|PIVOT-002 ready v\|PIVOT-003 ready v" .factory/specs/behavioral-contracts/BC-INDEX.md`

Results from table row section (lines 1–380, before changelog):
- Line 119: `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9` — CURRENT (no change)
- Line 120: `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.4` — STALE → fixed to `ready v2.9 (D-1114 2026-06-12)`
- No PIVOT-001/002/003 version pin annotations in BC-INDEX table rows — confirmed zero hits

Additional index sweeps:
- VP-INDEX: `grep -n "S-DEMO-DTU-LIVE-SCENARIO-001-B ready v\|PIVOT.*ready v" .factory/specs/verification-properties/VP-INDEX.md` — zero hits (VP-INDEX references story B by ID for traceability, no version pin annotations in table rows)
- ARCH-INDEX: `grep -n "S-DEMO-DTU-LIVE-SCENARIO-001-B ready v\|PIVOT.*ready v" .factory/specs/architecture/ARCH-INDEX.md` — zero hits (same; ID-only reference)
- STORY-INDEX row for story B: `ready v2.9` — CURRENT

**Story versions confirmed (from frontmatter):**
- S-DEMO-DTU-LIVE-SCENARIO-001-B: v2.9 — CURRENT
- S-DEMO-ENRICHMENT-PIVOT-001: v1.1
- S-DEMO-ENRICHMENT-PIVOT-002: v1.1
- S-DEMO-ENRICHMENT-PIVOT-003: v1.4

All PIVOT story versions match STORY-INDEX annotations. No stale version pins for PIVOT stories in BC-INDEX.

**Versions bumped:**
- BC-INDEX: v6.35→v6.36 (state-manager)

**Code:** Story B HEAD bc0f36c5 UNCHANGED (index-row annotation only; no BC semantic change; no code change required; no new push to PR #185).

**Lesson appended:**
- (z8) Shared-anchor-story index rows must be swept as a CLASS — when any burst bumps a story version, grep every index for ALL rows citing that story ID (not just the row for the BC being amended); same exhaustive-inventory principle as z5, applied to index annotations [process-gap].

---

## Do-Not-Reflag Addendum for Pass 9

All prior do-not-reflag entries from the pass-8 dispatch instructions carry forward. Add:

- **BPRL-P8-01 CLOSED:** BC-INDEX row 120 (BC-2.06.020) story-version pin synced v2.4→v2.9 (D-1114 2026-06-12). Exhaustive annotation sweep: VP-INDEX and ARCH-INDEX carry no version pin annotations; PIVOT-001/002/003 BC-INDEX rows carry no version pins. BC-INDEX v6.36. Story B HEAD bc0f36c5 UNCHANGED.

**Pass 9 ground truth:**
- Branch: `feature/S-DEMO-DTU-LIVE-SCENARIO-001-B`; REMOTE HEAD `bc0f36c5`; PR #185
- BC-2.06.019 is v1.7 — use the v1.7 Route Coverage Table (8 rows, exhaustive, corrected inventory note); do NOT cite v1.6 or earlier inventory-note prose
- BC-2.06.020 is v1.2 — anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9 (D-1114 2026-06-12)
- BC-INDEX v6.36; STORY-INDEX v2.362
