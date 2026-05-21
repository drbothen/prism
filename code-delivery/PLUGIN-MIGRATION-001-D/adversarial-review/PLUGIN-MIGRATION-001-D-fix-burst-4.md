---
document_type: fix-burst-closure-record
story_id: PLUGIN-MIGRATION-001-D
fix_burst_number: 4
pass_addressed: 4
closure_date: 2026-05-20
closure_decision: D-738
streak_status: 0/3 (unchanged; awaiting pass-5 fresh-context)
findings_total: 9
findings_closed: 8
findings_deferred: 1
deferred_items: [F-LP4-OBS-001 process-gap — codified as cycle-close discipline]
---

# PLUGIN-MIGRATION-001-D Fix-Burst-4 Closure

## Findings Closure Status

### HIGH (4/4 closed in-scope)

| Finding | Closure | Evidence |
|---|---|---|
| F-LP4-HIGH-001 (URL grounding — all 4 sensors 404 on DTU routes) | CLOSED | ADR-028 §D1 TOML spec URLs MUST ground against DTU clone route table, NOT production Rust adapter paths; BC-2.16.013 v1.4 §Postconditions updated with DTU-grounded URLs per sensor; HS-013/014/015/016 v1.1 URLs corrected; story v1.4 Task descriptions propagated |
| F-LP4-HIGH-002 (prism-sensors dev-dep contradiction) | CLOSED | ADR-028 §D3 — parity reference mechanism: committed fixture JSON (`crates/prism-dtu-{sensor}/fixtures/parity/reference-ocsf/<table>.json`) recorded from legacy adapter; no `prism-sensors` dev-dep required; BC-2.16.013 v1.4 §Parity Reference Mechanism updated; story v1.4 Task 10a (one-time fixture recording procedure) added |
| F-LP4-HIGH-003 (E-SPEC-017 implementation scope gap) | CLOSED | ADR-028 §D (Decision 3 mirror) — scope expansion authorized; story v1.4 Task 11 (add `SpecErrorCode::ESpec017` variant in `crates/prism-core/src/error.rs:892`); story v1.4 Task 12 (add filename-stem-vs-sensor_id check in `crates/prism-spec-engine/src/spec_parser.rs::load_all` at line 715); BC-2.16.001 v1.5 §Error Conditions updated — `SpecLoader::load_all()` emits E-SPEC-017 on mismatch, `SpecLoader::parse()` does not |
| F-LP4-HIGH-004 (Cyberint/Claroty auth_type DTU mismatch) | CLOSED | ADR-028 §D2 — auth_type ground against DTU enforcement, not `auth_type_name()` strings; Decision-4 LOCKED: cyberint=`cookie_roundtrip` (actual behavior per cyberint.rs:155 + DTU routes/alerts.rs:43-46), claroty=`bearer_static` (actual behavior per claroty.rs:63-65 + DTU enforcement); BC-2.16.013 v1.4 auth_type swap propagated; HS-013..016 v1.1 updated; story v1.4 AC-002/003/004/011 propagated |

### MED (3/3 closed in-scope)

| Finding | Closure |
|---|---|
| F-LP4-MED-001 (AC-001 "incidents is cursor" vs "2-step pipeline") | CLOSED — story v1.4 AC-001 corrected: incidents pipeline is 2-step (QueryV1→EntitiesV2) not cursor; "cursor" annotation removed |
| F-LP4-MED-002 (HS-018 cites `parse_spec_directory()` but RG-09 driver unnamed) | CLOSED — story v1.4 AC-001 / Task 12 / RG-09 driver explicitly named `SpecLoader::load_all` per spec_parser.rs conventions; HS-016 v1.1 corrected |
| F-LP4-MED-003 (BC `request_count == 2` fragile single-page assumption) | CLOSED — BC-2.16.013 v1.4 §Test Vectors: `request_count >= 2` relaxation (permits multi-page cursor continuation without test fragility) |

### LOW (1/1 closed)

| Finding | Closure |
|---|---|
| F-LP4-LOW-001 (AC examples use `unwrap()` inconsistent with style guidance) | CLOSED — story v1.4 §Style Guidance updated: `unwrap()` is explicitly permitted in test bodies (non-production code paths per CLAUDE.md error-handling rule); AC code samples annotated accordingly |

### OBS (1/1 — deferred to cycle-close, codified as discipline)

| Finding | Disposition |
|---|---|
| F-LP4-OBS-001 [process-gap] (POL-22 Phase C did not cross-check DTU clone routes in pass-1/2/3) | DEFERRED to cycle-close — codified as project discipline: "TOML-spec stories targeting DTU parity require dual code-grounding (production adapter code + DTU clone routes table)." Not blocking this cascade. |

## Cumulative Closures (All 4 Fix-Bursts)

| Burst | Pass Addressed | Findings Closed | Severity Breakdown |
|---|---|---|---|
| FB-IMPL-P1 (D-733) | Pass 1 | 14 | 5H + 3M + 4L + 2OBS |
| FB-IMPL-P2 (D-734) | Pass 2 | 10 | 3H + 3M + 2L + 2OBS |
| FB-IMPL-P3 (D-735) | Pass 3 | 12 | 3C + 2H + 1M + 6OBS |
| FB-IMPL-P4 (D-738) | Pass 4 | 9 | 4H + 3M + 1L + 1OBS-deferred |
| **TOTAL** | | **45** | **9H + 9M + 7L + 2C + 11OBS (1 deferred)** |

Note: Pass-3 findings were classified CRITICAL by adversary (per pass-3 report); recorded as C above for audit fidelity.

## DTU-EXT Items Surfaced (Flagged for Orchestrator Follow-Up)

These 4 items represent gaps in the DTU clone coverage exposed by ADR-028 grounding work. They are NOT blocking this cascade but require follow-up stories:

| ID | Description | Priority | Follow-up |
|---|---|---|---|
| DTU-EXT-001 | CrowdStrike DTU clone missing `/incidents` route — `prism-dtu-crowdstrike` has no `/detects/queries/incidents/v1` or `/detects/entities/incidents/POST/v1` routes | P1 | New story: extend prism-dtu-crowdstrike clone routes for incidents resource |
| DTU-EXT-002 | Claroty DTU clone route `POST /api/v1/devices` should also register `POST /api/v1/assets` alias (OR clarify canonical table name for Claroty assets) | P2 | ADR-028 §Known Gaps; PO + architect adjudication |
| DTU-EXT-003 | Cyberint DTU clone route is `GET /api/v1/alerts` but prior spec had `GET /api/alerts` (missing `/v1` segment) — DTU grounding per ADR-028 D1 resolves BC side; verify DTU route is real-API canonical | P2 | Verify against real Cyberint API docs; update DTU clone if needed |
| DTU-EXT-004 | Armis DTU clone uses separate `/api/v1/devices` and `/api/v1/alerts` routes; real Armis uses `/api/v1/search` with AQL `in:{table}` discriminator — DTU does NOT model the AQL discriminator pattern | P1 | New story: extend prism-dtu-armis clone to implement `/api/v1/search` with AQL routing |

## Architectural ADR Added

**ADR-028 — TOML Spec URL Grounding vs DTU Routes** (PROPOSED v1.0)
- Authored by: architect (FB-IMPL-P4 Step 1)
- Decision: TOML spec URLs, auth_type, and parity references MUST ground against DTU clone route table (real-API canonical), not production Rust adapter code
- §D1: URL grounding from DTU routes
- §D2: auth_type from DTU enforcement mechanism
- §D3: parity reference from committed fixture JSON
- §Known Gaps: DTU-EXT-001..004 documented
- ARCH-INDEX v2.85→v2.86 updated

## Scope Changes Summary

**Architect:** ADR-028 PROPOSED v1.0 + ARCH-INDEX v2.85→v2.86

**Product-Owner:**
- BC-2.16.013 v1.3→v1.4: URL re-grounding (DTU routes); fixture-JSON parity mechanism; auth_type swap (claroty=bearer_static, cyberint=cookie_roundtrip); `request_count >= 2` relaxation; §Known Gaps DTU-EXT-001..004
- BC-2.16.001 v1.4→v1.5: E-SPEC-017 enforcement contract — `SpecLoader::load_all()` emits, `SpecLoader::parse()` does not
- BC-INDEX v5.24→v5.25
- HOLDOUT-INDEX v1.4→v1.5
- HS-013/014/015/016 v1.0→v1.1
- TS-PLUGIN-PARITY-001 v1.0→v1.1

**Story-Writer:**
- PLUGIN-MIGRATION-001-D story v1.3→v1.4: Task 11 (ESpec017 variant), Task 12 (filename-stem check), Task 10a (fixture JSON recording); auth_type swap propagated; URL re-grounding propagated; AC-001 incidents 2-step pipeline; AC-007 `request_count >= 2` relaxation; RG-09 driver named; §Style Guidance unwrap()-permitted-in-tests; points 5→6
- STORY-INDEX v2.160→v2.161

**State-Manager (this burst):**
- fix-burst-4.md (this file)
- input-hash updated: story v1.4 `input-hash: "4e55025"` (SHA256 of sorted file-SHA pairs, first 7 chars, computed from current working tree content)
- STATE.md v7.424→v7.425

## Streak Status

- streak_before_pass4: 0/3
- streak_after_fb4: 0/3 (unchanged — this is a fix-burst, not an adversary pass)
- next_action: Pass-5 fresh-context adversary dispatch

## Next

Pass-5 with fresh-context adversary. Target streak 0/3 → 1/3 per BC-5.39.001 / D-716 Option A standing.
