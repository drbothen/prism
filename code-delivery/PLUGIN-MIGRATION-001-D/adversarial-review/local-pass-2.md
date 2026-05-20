---
document_type: adversarial-review-pass
story_id: PLUGIN-MIGRATION-001-D
pass_number: 2
pass_scope: LOCAL-SPEC-LEVEL
pass_date: 2026-05-20
adversary_model: Claude Opus 4.7 (1M context, fresh)
streak_before: 0/3
streak_after: 0/3
findings_summary: "3 HIGH + 3 MED + 2 LOW + 2 OBS (10 total)"
---

# PLUGIN-MIGRATION-001-D Pass-2 Adversarial Review (LOCAL Spec-Level)

## Scope
- Story spec PLUGIN-MIGRATION-001-D v1.1 (913 lines)
- BC-2.16.013 v1.1 (331 lines)
- HS-013, HS-014, HS-015, HS-016, HS-017, HS-018 (all 6 read in full)
- BC anchors: BC-2.01.013/v1.6, BC-2.01.016/v1.10, BC-2.16.001/v1.3, BC-2.16.002/v1.35, BC-2.16.009/v1.3, BC-2.16.012/v1.29
- TS-PLUGIN-PARITY-001, ADR-023, ADR-022
- Code parity sources: crates/prism-sensors/src/auth/{crowdstrike,claroty,cyberint,armis,mod}.rs; prism-spec-engine/src/{pipeline,spec_parser,auth_provider}.rs; prism-dtu-crowdstrike/src/clone.rs
- error-taxonomy.md v1.40 (E-SPEC-001..014)
- Indices: BC-INDEX v5.22, STORY-INDEX v2.158, HOLDOUT-INDEX v1.4, VP-INDEX

## Methodology
Two-phase POL-22 verification: Phase A lexical citation match + Phase C named-entity existence. Code-grounded read of all Rust symbol claims.

## Findings

### HIGH

#### F-001 [CRITICAL HIGH] PolicyViolation:POL-4,POL-22,Standing Rule 3 §1 — Cyberint and Claroty `auth_type` SWAPPED in spec vs code
**Locations:** Story lines 159-160, 241-242, 255-256, 442-445; BC-2.16.013 lines 123, 131; HS-014, HS-015.
**Evidence:** `crates/prism-sensors/src/auth/cyberint.rs:57-59` → `auth_type_name() = "bearer_static"`; `claroty.rs:63-65` → `auth_type_name() = "cookie_roundtrip"`. mod.rs:13-14 docs confirm. Spec asserts OPPOSITE.
**Routing:** product-owner.

#### F-002 [HIGH] PolicyViolation:POL-22 Phase C, POL-4, POL-24 — E-SPEC-009 phantom semantics for filename-stem mismatch
**Locations:** BC-2.16.013 §Error Conditions line 256; story AC-001 line 224, RG-09 line 503; HS-018.
**Evidence:** error-taxonomy.md:381 — E-SPEC-009 = "Duplicate sensor_id across spec files". No filename-stem check exists in spec_parser.rs::load_all_specs. Pass-1 retired E-SPEC-016 but "replaced" with semantically-wrong E-SPEC-009.
**Routing:** product-owner (author new code per POL-1 append-only).

#### F-003 [HIGH] PolicyViolation:POL-22 Phase C, POL-31 — `CrowdStrikeAdapter::fetch_page()` phantom symbol
**Locations:** BC-2.16.013 §Postconditions §1 line 120; story AC-007 line 350; HS-013-01 line 72.
**Evidence:** grep `fetch_page` returns no matches in crates/. Actual methods: `fetch_entities` (line 305), `fetch` SensorAdapter trait impl (line 391).
**Routing:** product-owner.

### MED

#### F-004 [MED] PolicyViolation:POL-29 — `${query.aql}` survives in BC-2.16.013 §Canonical Test Vectors line 276 (pass-1 sibling-sweep gap)
**Routing:** product-owner (1-line fix, BC v1.1 → v1.2).

#### F-005 [MED] TD-VSDD-091 — Line-number citations in BC + story Task 1 (`spec_parser.rs:128`, `pipeline.rs:246-250`)
**Routing:** product-owner (symbol-name replacement).

#### F-006 [MED] PolicyViolation:POL-29, POL-4 — 6 new HS files use `epic_id: "E-PLUGIN-MIGRATION"` vs sibling convention `PLUGIN-MIGRATION-001`
**Routing:** product-owner (align 6 files).

### LOW

#### F-007 [LOW] PolicyViolation:POL-23 — BC-2.16.009 §Error Conditions doesn't enumerate E-SPEC-002, E-SPEC-003 (HS-017 expects them; canonical taxonomy has them)
**Routing:** product-owner (BC-2.16.009 v1.3 → v1.4).

#### F-008 [LOW] — AC-011 claims 4-value auth_type set; canonical is 5-value (+ `custom_via_plugin`)
**Routing:** Bundled with F-007 BC-2.16.009 amendment.

### OBS

#### O-001 — Pass-1 closed 7 of 14 findings in-scope but pass-2 found 3 NEW HIGH. Fresh-context value confirmed; novelty HIGH.
#### O-002 — VP-148 indexed-only-no-file (known process-gap; not blocking).

## Verdict
**BLOCKED-soft.** 10 findings. Streak 0/3.

## Routing Summary
All findings (F-001..F-008) route to product-owner. Cyberint code-side label-vs-behavior inconsistency surfaced as code-side tech-debt for cycle-close architect/implementer review.

## Novelty Assessment
**HIGH.** 3 critical defects pass-1 did not catch: auth_type swap (Phase-C verification of `auth_type_name()` strings), E-SPEC-009 phantom semantics (deep error-taxonomy + spec_parser audit), `fetch_page` phantom symbol. Confirms fresh-context compounding value principle.

## Streak Update
- streak_before: 0/3
- streak_after: 0/3
- next_action: FB-IMPL-P2 (PO + story-writer) → pass-3 with fresh context
