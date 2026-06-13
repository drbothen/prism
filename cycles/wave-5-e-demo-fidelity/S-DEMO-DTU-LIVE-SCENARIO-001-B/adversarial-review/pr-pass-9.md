---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr_number: 185
pass_number: 9
cascade: PR-LEVEL (distinct from LOCAL; LOCAL CONVERGED 3/3 strict @13 passes)
base_develop: "939f36ce"
feature_head_at_review: "bc0f36c5"
feature_head_after_fix_burst: "bc0f36c5"
clean_strict: true
clean_pr_merge: true
streak_after: "1/3"
produced: 2026-06-12
authority: BC-5.39.001 D-779
decision: D-1115
---

# PR-LEVEL Adversary Pass 9 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B — Scenario Progression + Enrichment Correlation Live Demo
**PR:** #185 (base develop@939f36ce, head bc0f36c5 — unchanged from pass 8; no code change since pass 4)
**Pass:** PR-LEVEL pass 9 (distinct from LOCAL cascade; LOCAL CONVERGED 3/3 strict at 13 passes)
**Date:** 2026-06-12

## Pass-8 Closure Verification

All pass-8 findings verified sound:

- **BPRL-P8-01 MED [process-gap]** (BC-INDEX row 120 story-version pin stale v2.4 after D-1113 story B v2.9 advance): CLOSED. BC-INDEX row-120 annotation corrected `ready v2.4 (B-P5-03 2026-06-12)` → `ready v2.9 (D-1114 2026-06-12)`. Exhaustive annotation sweep confirmed VP-INDEX and ARCH-INDEX carry no version pin annotations for story B; PIVOT-001/002/003 BC-INDEX rows carry no version pins. BC-INDEX v6.36. Story B HEAD bc0f36c5 UNCHANGED (index-row annotation only). **VERIFIED — BC-INDEX row 120 now reads `ready v2.9 (D-1114 2026-06-12)`. CLOSED stands.**

## Pass-9 Verification Axes

### Axis 1: BC-INDEX version pins (all story-B-anchored rows)

Sweep: both row 119 (BC-2.06.019) and row 120 (BC-2.06.020) — PASS.
- Row 119: `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9 (D-1113 2026-06-12)` — CURRENT
- Row 120: `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9 (D-1114 2026-06-12)` — CURRENT
- STORY-INDEX story B row: `ready v2.9` — CURRENT
- BC-2.06.019 frontmatter version: v1.7 — matches BC-INDEX row 119 pin PASS
- BC-2.06.020 frontmatter version: v1.2 — matches BC-INDEX row 120 pin PASS

### Axis 2: BC-2.06.019 v1.7 Route Coverage Table integrity

8-row EXHAUSTIVE table verified. All StageMask-guarded route files in PR diff accounted for:
1. `crates/prism-dtu-crowdstrike/src/routes/hosts.rs` — `GET /devices/entities/devices/v2` — guarded `mask.lateral_devices`
2. `crates/prism-dtu-crowdstrike/src/routes/detections.rs` — `GET /detects/queries/detects/v1` — guarded `mask.detections`
3. `crates/prism-dtu-crowdstrike/src/routes/summaries.rs` — `POST /detects/entities/summaries/GET/v1` — guarded `mask.detections`
4. `crates/prism-dtu-armis/src/routes/devices.rs` — `GET /api/v1/devices` — guarded `mask.lateral_devices`
5. `crates/prism-dtu-armis/src/routes/alerts.rs` — `GET /api/v1/alerts` — guarded `mask.detections`
6. `crates/prism-dtu-armis/src/routes/search.rs` — `GET /api/v1/search` — UNGUARDED (returns empty when no stage context; per-spec behavior)
7. `crates/prism-dtu-cyberint/src/routes/alerts.rs` — `GET /api/v2/alerts` — guarded `mask.detections` (StageMask projection implemented; BPRL-P2-01 closed)
8. `crates/prism-dtu-claroty/src/routes/devices.rs` — `GET /api/v2/devices` — guarded (StageMask-guarded; AC-015 load-bearing; BPRL-P6-01 closed)

claroty/alerts.rs: EXEMPT — real-API grounds (Claroty alerts endpoint does not support server-side stage filtering); EXEMPT note correctly states claroty/alerts.rs does NOT appear in either grep set (verified v1.7 correction from BPRL-P7-01 closure). PASS.

### Axis 3: BC-2.06.020 invariant consistency

Enrichment correlation BC content v1.2 reviewed. All invariants consistent:
- Enrichment query correlation with scenario IOC surface defined
- E-DEMO-006 error taxonomy reference present and byte-exact per error-taxonomy v1.78
- Frontmatter: story anchor S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9 (D-1114) — matches BC-INDEX row 120 PASS

### Axis 4: E-DEMO error taxonomy — verbatim message consistency

E-DEMO-006 message in code, BC-2.06.019, and error-taxonomy v1.78 compared verbatim. All three references identical. PASS.

### Axis 5: SAP-1 — Structured Event Catalog completeness

`rg 'event_type\s*=' crates/ --type rust` sweep over PR diff. No new `event_type` emissions introduced by Story B branch (diff unchanged since pass 4; SAP-1 has been PASS for passes 5–8 inclusive). PASS.

### Axis 6: SAP-2 — DTU↔TOML schema parity

No sensor TOML files (`*.sensor.toml` under `.prism/specs/sensors/`) modified in PR diff. N/A — not applicable.

### Axis 7: Forbidden-pattern sweep

- `unwrap()` / `expect()` on `Result` in non-test paths: NONE introduced by Story B diff
- `println!` in production code: NONE
- `reqwest::Client::new()` without `.timeout()`: NONE
- Retired `ColumnType` variants (Int64/Float64/Timestamp from prism_spec_engine::types): NONE
- `Arc::new(SomeThing::placeholder())` stub construction: NONE
All forbidden patterns: PASS.

### Axis 8: DormantTenant regression guard

Red Gate test 17 (DormantTenant isolation guard) present and non-vacuous in Story B test suite. PASS.

### Axis 9: Demo evidence completeness

18/18 ACs confirmed COMPLETE in demo evidence at `docs/demo-evidence/S-DEMO-DTU-LIVE-SCENARIO-001-B/`. Demo evidence commit range intact (recording-HEAD 785adc4b — benign, recorded before final fixups; evidence matches spec intent; do-not-reflag per pass-4 adjudication). PASS.

### Axis 10: Frontmatter-body coherence

Story B frontmatter: `acceptance_criteria_count: 18`, `red_gate_tests: 19` consistent with AC body enumeration and test suite. BC-2.06.019 v1.7 frontmatter pin in story body matches. BC-2.06.020 v1.2 frontmatter pin in story body matches. PASS.

### Axis 11: Story B HEAD consistency

`git -C .worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B log -1 --format='%H %s'` → `bc0f36c5` = remote (no code change since pass 4). PR diff unchanged. PASS.

### Axis 12: BC-2.06.019 v1.7 frontmatter-body coherence

BC frontmatter version v1.7 consistent with body changelog (v1.6→v1.7 entry present: fabricated inventory-note prose corrected per BPRL-P7-01). Route Coverage Table row count 8 in body matches body narrative "8-row EXHAUSTIVE". Inventory note for claroty/alerts.rs correctly states EXEMPT on real-API grounds without fabricated grep claim. PASS.

### Axis 13: Story B v2.9 — all BC pins consistent

BC-2.06.019 pin in story body: v1.7 — matches BC file frontmatter PASS.
BC-2.06.020 pin in story body: v1.2 — matches BC file frontmatter PASS.
ADR-036 pin in story body: v2.3 — consistent with ADR-036 current version PASS.

### Axis 14: Novelty check — any new patterns not covered by do-not-reflag list

Full do-not-reflag list for pass 9 reviewed (all LOCAL closures + BPRL-P1 through BPRL-P8). No novel finding classes identified. Code has been stable since pass 4 (bc0f36c5). All spec-side corrections from passes 5–8 are well-attested and verifiably closed. Novelty assessment: LOW — no new patterns.

### Axis 15: Cross-record consistency — BC-INDEX / STORY-INDEX / VP-INDEX / ARCH-INDEX

- BC-INDEX v6.36: total 250; active 232; draft 5; retired 6 — consistent with STATE.md frontmatter PASS
- STORY-INDEX v2.362: total_stories 200 — consistent with STATE.md frontmatter PASS
- VP-INDEX v1.79: 158 registered — consistent with STATE.md frontmatter PASS
- ARCH-INDEX v2.133 — no story-B version pins; consistent PASS

## Pass-9 Finding

**ZERO findings of any severity.**

No CRITical findings.
No HIGH findings.
No MED findings.
No LOW findings.
No OBS findings.
No PROCESS-GAP findings.

All 15 verification axes passed. All prior pass closures verified sound and propagated correctly. Spec and implementation are fully converged. The PR is spec-sanctioned and production-grade.

**CLEAN(strict):** YES — zero findings of ANY severity (streak criterion met)
**CLEAN(PR-merge):** YES — zero CRIT+HIGH+MED findings
**Streak:** 1/3

---

## Convergence Summary

The PR-LEVEL cascade has reached streak 1/3 under BC-5.39.001 3-CLEAN strict protocol. Pass-9 is the first clean strict pass of the PR-LEVEL cascade (passes 1–8 each had at least one finding of some severity; pass 9 is the first zero-finding pass of any severity).

**Assessment:** Spec/implementation convergence confirmed. The route coverage table (BC-2.06.019 v1.7, 8-row EXHAUSTIVE) is accurate and load-bearing. All IOC-surface, enrichment-correlation, and error-taxonomy references are internally consistent. The do-not-reflag list comprehensively accounts for all adjudicated decisions (BPRL-P4-01 IOC-surface deferral, Armis key-presence discriminator, BC stage-0 tension, historical evidence-report citations). Code is production-grade for Story B's defined scope.

**NEXT:** PR-LEVEL pass 10 at HEAD bc0f36c5 (diff unchanged). Dispatch fresh adversary. Do-not-reflag list: all prior entries plus BPRL-P9 (pass 9 CLEAN — no new do-not-reflag entries needed as pass produced zero findings).

---

## Do-Not-Reflag Addendum for Pass 10

All prior do-not-reflag entries from the pass-9 dispatch instructions carry forward. No new entries needed (pass 9 produced zero findings — nothing to add).

**Pass 10 ground truth:**
- Branch: `feature/S-DEMO-DTU-LIVE-SCENARIO-001-B`; REMOTE HEAD `bc0f36c5`; PR #185
- BC-2.06.019 is v1.7 — use the v1.7 Route Coverage Table (8 rows, exhaustive, corrected inventory note; claroty/alerts.rs EXEMPT on real-API grounds); do NOT cite v1.6 or earlier inventory-note prose
- BC-2.06.020 is v1.2 — anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.9 (D-1114 2026-06-12)
- BC-INDEX v6.36; STORY-INDEX v2.362
- Pass 9 result: CLEAN(strict)=YES; CLEAN(PR-merge)=YES; streak 1/3
