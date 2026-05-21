---
document_type: fix-burst-closure-record
story_id: PLUGIN-MIGRATION-001-D
pass_number: 13
closure_date: 2026-05-20
findings_total: 4
findings_closed: 4
findings_deferred: 0
streak_before: 0/3
streak_after: 0/3
---

# Fix-Burst-13 Closure Record

## Summary

4 findings closed in-scope (1 HIGH + 2 MED + 1 LOW process-gap) per user Path A adjudication (D-747). Pass-13 surfaced the 6th novel coherence-axis class: inter-ADR contradiction with shipped+tested code witness.

## Per-Finding Closure Table

| Finding | Severity | Closure Action | Scope |
|---------|----------|---------------|-------|
| F-LP13-HIGH-001 | HIGH | ADR-026 §D3 ↔ ADR-028 §D2 bidirectional Supersedes linkage; ADR-028 v1.4 → v1.5 (§D2 supersession prefix; new §D6 documents PLUGIN-MIGRATION-001-A scope expansion — auth_type_name() rewrites for Cyberint/Claroty/Armis + Red Gate test_BC_2_01_016_003 amendment in 001-A scope); ADR-026 v1.29 → v1.30 (Superseded-by ADR-028 §D2 partial; §D3 supersession prefix with migration-window note); ARCH-INDEX v2.90 → v2.91. User Path A locked at D-747. | architect |
| F-LP13-MED-001 | MED | Story AC-001 + Task 3 reconciled per ADR-028 §D5: include incidents with DTU-EXT-001 gap (mandate not "may omit"); tables.len() == 3 retained in AC-001 PASS criterion; Task 3 rewritten to mandate include-with-gap; Task 4/5/6 ADR-028 §D6 footnotes added; BC-2.16.013 v1.7 pin swept across 6 active-prose sites; story v1.6 → v1.7. | story-writer |
| F-LP13-MED-002 | MED | HS-018 v1.0 → v1.1: §Evaluation Criteria clarifies HS-018-02 (case-mismatch CrowdStrike vs crowdstrike) covered by RG-09 case-sensitive byte-equality (Option A; no new RG needed); HOLDOUT-INDEX v1.9 → v1.10. | product-owner |
| F-LP13-LOW-001 | LOW | TS-PLUGIN-PARITY-001 frontmatter `modified: "2026-05-20"` added per POL-27 extension (missing field = stale-equivalent). BC-2.16.013 v1.7 pin propagated into §Architecture Anchors; §Postconditions §1 Cyberint/Claroty/Armis rows annotated with supersession context; BC-INDEX v5.28 → v5.29. | product-owner |

## Artifact Version Deltas

| Artifact | Before | After |
|----------|--------|-------|
| ADR-028-toml-spec-grounding-vs-dtu-routes.md | v1.4 | v1.5 |
| ADR-026-sensorauth-unsealing.md | v1.29 | v1.30 |
| ARCH-INDEX.md | v2.90 | v2.91 |
| BC-2.16.013-bundled-sensor-spec-dtu-parity.md | v1.6 | v1.7 |
| BC-INDEX.md | v5.28 | v5.29 |
| HS-018-spec-id-filename-mismatch-rejection.md | v1.0 | v1.1 |
| HOLDOUT-INDEX.md | v1.9 | v1.10 |
| TS-PLUGIN-PARITY-001-dtu-canonicalization.md | — | modified field added |
| PLUGIN-MIGRATION-001-D story | v1.6 | v1.7 |
| STORY-INDEX.md | v2.164 | v2.165 |
| local-pass-13.md | — | new |
| PLUGIN-MIGRATION-001-D-fix-burst-13.md | — | new |

## Cumulative Closures

59 (through fix-burst-12) + 4 = **63 across 13 fix-bursts**.

## Streak

- Before: 0/3
- After: 0/3 (streak still reset; pass-14 fresh-context dispatch pending)

## Lesson

**6th novel coherence-axis class** caught by fresh-context pass-13 after 12 prior passes treated ADR-026 §D3 as invisible: inter-ADR contradiction with shipped+tested code witness. ADR-026 §D3 (auth_type_name() return strings mandated ACTIVE in develop@1bc56e3c via PR #151) directly contradicted ADR-028 §D2 (those same strings labeled "latent label bugs") with no Supersedes linkage between them; Red Gate test_BC_2_01_016_003 provided the code witness.

**S-7.02 codification candidate:** Adversary must cross-check every ADR claim against ALL active ADRs (not just the one under review) and against shipped+tested code (third-party witness). This pattern observed across passes 9/10/11/12/13 — five novel coherence-axis classes in five consecutive passes suggests a systematic scope gap in earlier passes.

## User Decision

D-747 Path A locked: ADR-028 explicitly supersedes ADR-026 §D3 (partial — auth_type_name() return values for Cyberint/Claroty/Armis); PLUGIN-MIGRATION-001-A scope EXPANDS to include rewriting these auth_type_name() returns + amending Red Gate test_BC_2_01_016_003. CrowdStrike unchanged.
