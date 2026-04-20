---
document_type: adversarial-review
level: ops
version: "1.0"
status: findings-open
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs:
  - .factory/specs/prd.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/prd-supplements/test-vectors.md
  - .factory/specs/prd-supplements/error-taxonomy.md
  - .factory/specs/domain-spec/invariants.md
  - .factory/specs/verification-properties/VP-INDEX.md
  - .factory/stories/STORY-INDEX.md
input-hash: "[commit-64dfbba]"
traces_to: prd.md
cycle: phase-2-patch
pass: 27
previous_review: adversarial-reviews/pass-26.md
novelty: HIGH
findings: 9
critical: 2
high: 4
medium: 2
low: 1
previous_pass: 26 (15 findings: 7 HIGH, 6 MED, 2 LOW — 14 closed Burst 27, 1 M-006 closed via clarifying note)
convergence_counter: 0 of 3
---

# Pass 27 — Fresh-Context Review with New test-vectors.md Supplement

## Finding ID Convention

Finding IDs in this pass use the project-local format: `P3P27-A-<SEV>-<SEQ>`

- `P3P27`: Phase-2-patch cycle, pass 27
- `<SEV>`: `C` (CRITICAL), `H` (HIGH), `M` (MEDIUM), `L` (LOW)
- `<SEQ>`: three-digit sequence

Template-canonical equivalent: `ADV-P2PATCH-P27-<SEV>-<SEQ>`

## Scope Note

Fresh-context pass against the full spec+story package at commit `64dfbba` after Burst 27. Primary new scope: `prd-supplements/test-vectors.md` (v1.0, 10 TVs) created this burst to close pass-26 H-006. Secondary: Burst 27 closure verification for the 15 pass-26 findings; Wave-1/2/4/5/6 BC-title sweep; DI citation bidirectional check; VP-INDEX / STORY-INDEX / BC-INDEX arithmetic. Did not read prior pass findings.

## Part A — Fix Verification (Burst 27 Closure)

| Pass-26 ID | Severity | Claim | Verified? | Evidence |
|-----------|----------|-------|-----------|----------|
| H-001 | HIGH | S-4.06 AC-13 `[PHASE 3 PATCH]` stripped; no `[PHASE 3 PATCH` remains | MOSTLY | No literal `[PHASE 3 PATCH` in stories/; S-4.06 still has `[SCOPE EXPANSION — Phase 3 patch]` at line 183, S-4.03 at line 156. Marker variant not covered by pass-26 H-001 scope. |
| H-002 | HIGH | S-4.07 lines 47-49 BC-2.14.008/.010/.012 titles canonical | YES | S-4.07 lines 47-49 titles match BC-INDEX v4.9 verbatim |
| H-003 | HIGH | S-4.02/.04/.05 9 BC titles canonical | YES | S-4.02 lines 46-50, S-4.04 lines 47-51, S-4.05 line 47 match BC-INDEX |
| H-004 | HIGH | S-3.02 line 53 virtual fields backticked correctly | YES | S-3.02 line 53 has `` `_sensor`, `_client`, `_source_table` `` |
| H-005 | HIGH | S-1.08 BC-2.04.005 canonical title | YES | S-1.08 line 42 title matches BC-INDEX |
| H-006 | HIGH | test-vectors.md created with 10 TVs; PRD §5b present; supplements[] includes it | YES (file exists) / NEW ISSUES | File exists with 10 TVs; PRD frontmatter line 12 includes it. BUT multiple TVs have fabricated content drifting from BC source of truth — see P3P27-A-C-001 and P3P27-A-C-002. |
| H-007 | HIGH | 7 DIs (016/025/027/028/029/030/031) each cited in at least one BC | YES | DI-016→BC-2.05.004; DI-025→BC-2.14.002; DI-027→BC-2.15.007; DI-028→BC-2.12.001/BC-2.13.006; DI-029→BC-2.06.005; DI-030→BC-2.16.001/.005/.007/.009; DI-031→BC-2.16.005/.007 |
| M-001 | MED | 4 SS-16 BCs have canonical `## Traceability` tables | YES | BC-2.16.001 line 64-70, BC-2.16.005 line 98-105, BC-2.16.007 line 64-71, BC-2.16.009 line 81-88 all use `## Traceability` table |
| M-002 | MED | S-1.09 BC-2.04.009 title canonical | YES | S-1.09 line 40 matches BC-INDEX |
| M-003 | MED | S-3.02 BC-2.11.001 has backticks on `query` | YES | S-3.02 line 48: `` BC-2.11.001 | `query` MCP Tool ... `` |
| M-004 | MED | BC-INDEX Subsystem Summary has split Removed+Retired columns; SS-01=6/0, SS-12=0/2 | YES | BC-INDEX lines 235-257 show split columns; SS-01 `6|0`, SS-12 `0|2` |
| M-005 | MED | S-4.03 Task 8a AND AC-9 reconciled to BC-2.13.014 (100k/10MB) | YES | S-4.03 Task 8a lines 183-186 and AC-9 lines 240-244 cite BC-2.13.014 values |
| M-006 | MED | BC-INDEX total_contracts clarifying note present | YES | BC-INDEX lines 19-23 include the clarifying note |
| L-001 | LOW | S-1.14/.15/S-4.08 `[PHASE 3 PATCH]` markers removed | PARTIAL | Literal `[PHASE 3 PATCH]` gone; `[SCOPE EXPANSION — Phase 3 patch]` variants remain |
| L-002 | LOW | S-4.08 BC table has `| BC ID | Title | Clause/Invariant |` 3-col schema | YES | S-4.08 line 49 has that schema with 9 populated rows |

**Net Burst 27 closure:** 14 of 15 pass-26 findings fully landed. H-006 landed as file creation but introduced fresh content-integrity defects (tracked below).

---

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

#### P3P27-A-C-001 — TV-006 uses wrong state names and wrong error code versus BC-2.14.002 source of truth

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** `.factory/specs/prd-supplements/test-vectors.md` lines 211-234
- **Policy violated:** 4 (`semantic_anchoring_integrity`), 7 (`bc_h1_is_title_source_of_truth`)
- **Novelty:** NEW
- **Description:** TV-006 declares case states as `New, In_Progress, Contained, Resolved, False_Positive` (line 215) and invalid-transition error code `E-CASE-003` (line 232). BC-2.14.002 source of truth (`behavioral-contracts/BC-2.14.002-case-state-transitions.md` lines 26, 30-48) declares states as `New, Acknowledged, Investigating, Resolved, Closed` and line 68 declares `E-CASE-004` for invalid transitions. error-taxonomy.md lines 240-242 corroborate (E-CASE-004 = invalid, E-CASE-005 = self-transition, E-CASE-006 = missing disposition). S-4.06 (lines 87-93) and DI-025 (invariants.md line 45) independently agree with the BC.
- **Evidence:** TV-006's transition matrix is fabricated: rows like `New→Contained`, `Contained→In_Progress`, `False_Positive (terminal)` do not exist in the BC. TV-006 also confuses state names with disposition codes — `FalsePositive` is a `DispositionCode`, not a state. A test-writer consuming TV-006 would implement a state machine against a nonexistent enum, use an error code that error-taxonomy assigns to "disposition required" (not invalid transition), and contradict VP-005/VP-006 (which enforce the real 12-transition machine). Policy 4 and Policy 7 both violated — the TV actively misrepresents the anchor BC.
- **Proposed Fix:** Rewrite TV-006 states to `New, Acknowledged, Investigating, Resolved, Closed`; rewrite transition matrix to BC-2.14.002 lines 30-48 (12 valid transitions); change `E-CASE-003` to `E-CASE-004`.

---

#### P3P27-A-C-002 — TV-002 uses wrong token TTL, wrong UUID version, wrong (removed) error code vs BC-2.04.009

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** `.factory/specs/prd-supplements/test-vectors.md` lines 88-115
- **Policy violated:** 4 (`semantic_anchoring_integrity`), 7
- **Novelty:** NEW
- **Description:** Three independent values in TV-002 contradict their anchor BC and cross-references.
- **Evidence:**
  - Line 106: `"expires_at": "<GENERATED:ISO8601-UTC+15m>"` — BC-2.04.009 line 39 says `expires_at: created_at + 300s (5 minutes)`; DI-007 says "valid for exactly 300 seconds".
  - Line 105: `"confirmation_token": "<GENERATED:UUID-v4>"` — S-1.09 Task 3 (line 49) specifies UUID **v7**, not v4.
  - Line 113: cap-exceeded error is `E-CONFIRM-001` — BC-2.04.009 line 55 says `E-FLAG-007`. error-taxonomy.md line 270 explicitly states `E-CONFIRM-001` is REMOVED (cases covered by E-FLAG-003 and E-FLAG-008 per DEC-009/016).
  - Grep confirms `E-CONFIRM-001` appears ONLY in test-vectors.md — nowhere else in the corpus.
  - A test fixture asserting 15-minute TTL directly contradicts VP-007 and DI-007 (which enforce 300s expiry boundary). Using a removed error code means the emitted error would never match the test assertion; worse, a test-writer might "resurrect" E-CONFIRM-001 to pass the test. UUID v4 when S-1.09 mandates v7 creates mismatched type assertions.
- **Proposed Fix:** Change `ISO8601-UTC+15m` → `ISO8601-UTC+5m`; change `<GENERATED:UUID-v4>` → `<GENERATED:crypto-random>` (or mirror BC field names token_id/action_hash/expires_at); change `E-CONFIRM-001` → `E-FLAG-007`.

---

### HIGH

#### P3P27-A-H-001 — S-1.14 and S-1.15 body BC tables use non-canonical schema (Wave-1 sweep drift)

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `.factory/stories/S-1.14-infusion-specs.md` lines 48-54; S-1.15 lines 48-55
- **Policy violated:** 7 (`bc_h1_is_title_source_of_truth`), 8 (`bc_array_changes_propagate_to_body_and_acs`)
- **Novelty:** NEW (Wave 1 drift not flagged in prior passes)
- **Description:** S-1.14 body BC table (lines 48-54) has columns `| BC | Invariant | Description |` — canonical BC title column absent. Descriptions are paraphrases, not BC H1 titles (e.g., BC-INDEX says `BC-2.19.001 | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF`). Same pattern in S-1.15 vs BC-INDEX lines 212-217. S-4.08 was explicitly converted to 3-col `| BC ID | Title | Clause/Invariant |` during Burst 27; S-1.14/S-1.15 remain on a fourth schema that omits canonical Title entirely. Grep: only 2 stories use this schema; 34 use canonical `| BC ID | Title`.
- **Evidence:** Implementer cannot reliably map BCs without cross-referencing. Burst 27 STORY-INDEX burst log claims `systematic Wave-1-5 BC title sweep across S-1.08/.09/.14/.15` but only frontmatter and AC traces were touched for S-1.14/S-1.15 — body tables missed.
- **Proposed Fix:** Convert S-1.14 and S-1.15 body BC tables to canonical `| BC ID | Title |` or S-4.08 3-col `| BC ID | Title | Clause/Invariant |` schema with BC-INDEX v4.9 titles verbatim.

---

#### P3P27-A-H-002 — TV-010 traceability matrix attributes DI-031 to BC-2.16.001; BC-2.16.001 does not cite DI-031

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `.factory/specs/prd-supplements/test-vectors.md` line 378
- **Policy violated:** 4 (`semantic_anchoring_integrity`), 2 (`lift_invariants_to_bcs`)
- **Novelty:** NEW
- **Description:** TV-010 traceability matrix row collapses multi-anchor TV under BC-2.16.001 only, falsely attributing DI-031 to that BC.
- **Evidence:** Line 378: `| TV-010 | BC-2.16.001 | DI-030, DI-031 | VP-023 |`. BC-2.16.001 body line 68: `| L2 Invariants | DI-008, DI-030 |` — DI-031 NOT cited. DI-031 is enforced by BC-2.16.005 and BC-2.16.007. TV-010 Trace line (361) lists BC-2.16.001 + BC-2.16.007 + DI-030 + DI-031; but traceability matrix collapses under BC-2.16.001 only. Consumer reading matrix would believe BC-2.16.001 enforces DI-031 and may add a citation, contaminating the source BC. Exactly the drift pattern Policy 2's bidirectional check exists to catch.
- **Proposed Fix:** Split TV-010 row into `BC-2.16.001 | DI-008, DI-030` and `BC-2.16.007 | DI-030, DI-031`, or clarify as multi-anchor vector. Do NOT strike DI-031 — the hot-reload scenario is real content.

---

#### P3P27-A-H-003 — S-1.09 uses `E-FLAG-002` for token expiry; error-taxonomy assigns `E-FLAG-003`

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `.factory/stories/S-1.09-confirmation-tokens.md` lines 51, 62; error-taxonomy.md lines 70-71
- **Policy violated:** 4 (`semantic_anchoring_integrity`)
- **Novelty:** NEW
- **Description:** S-1.09 Task 5 and AC-3 use E-FLAG-002 for "token expired" — but the taxonomy assigns E-FLAG-002 to a compile-time feature gate error and E-FLAG-003 to token expiry.
- **Evidence:** S-1.09 Task 5 (line 51): `E-FLAG-002: "Token expired..."`. S-1.09 AC-3 (line 62): `Then E-FLAG-002 is returned (BC-2.04.011)`. error-taxonomy.md line 70 assigns `E-FLAG-002` to "Write capability not compiled (cargo feature absent)" — compile-time feature gate. Line 71 assigns `E-FLAG-003` to "Token expired for action '{action_summary}'". BC-2.04.011 ("Token Expiry at 300 Seconds with Structured Error Recovery") semantically matches `E-FLAG-003`. Implementer emits `E-FLAG-002` for expired tokens; E2E tests assert `E-FLAG-002` per story; but taxonomy consumers, audit, agent retry logic expect `E-FLAG-003`. Two code paths emit conflicting codes for the same condition.
- **Proposed Fix:** Update S-1.09 Task 5 and AC-3: `E-FLAG-002` → `E-FLAG-003`.

---

#### P3P27-A-H-004 — S-2.01 body BC-2.15.002 title dropped canonical `removeRange` and `per Domain` tokens

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `.factory/stories/S-2.01-rocksdb-init.md` line 46
- **Policy violated:** 7 (`bc_h1_is_title_source_of_truth`)
- **Novelty:** NEW (Wave 2 sample drift — Wave 2 not in Burst 27 sweep scope)
- **Description:** S-2.01 carries a stale BC-2.15.002 title that omits `removeRange` and `per Domain` — both added in BC-INDEX v4.6 and present in BC-INDEX v4.9.
- **Evidence:** S-2.01 line 46: `| BC-2.15.002 | Domain-Based Key-Value Operations — get/put/putBatch/remove/scan |`. BC-INDEX v4.9 line 192: `| BC-2.15.002 | Domain-Based Key-Value Operations — get/put/putBatch/remove/removeRange/scan per Domain |`. BC-INDEX §4.6 changelog (line 455) records: `BC-2.15.002: Added "removeRange" (was dropped from BC-INDEX); added "per Domain"`. Story did not propagate v4.6 sync. Wave 2 was not in Burst 27 sweep list; systematic Wave-2 drift surfaces on first sample.
- **Proposed Fix:** Update S-2.01 line 46 to match BC-INDEX verbatim.

---

### MEDIUM

#### P3P27-A-M-001 — BC-2.16.001 and BC-2.16.009 Priority disagree between body (P1) and BC-INDEX (P0)

- **Severity:** MEDIUM
- **Category:** contradictions
- **Location:** BC-2.16.001 line 70; BC-2.16.009 line 88; BC-INDEX lines 202, 210
- **Policy violated:** 4 (`semantic_anchoring_integrity`), 7
- **Novelty:** NEW
- **Description:** BC-2.16.001 and BC-2.16.009 body Traceability tables say Priority=P1; BC-INDEX says both are P0. The two other SS-16 BCs (BC-2.16.005, BC-2.16.007) agree body=index at P1, so this is not a global pattern — it is specific to these two.
- **Evidence:**
  - BC-2.16.001 body Traceability table line 70: `| Priority | P1 |`
  - BC-INDEX v4.9 line 202: `BC-2.16.001 | Sensor Spec File Loading | ... | P0 | draft`
  - BC-2.16.009 body line 88: `| Priority | P1 |`
  - BC-INDEX v4.9 line 210: `BC-2.16.009 | ... | P0 | draft`
  - BC-2.16.005 (body P1 / index P1 — agrees), BC-2.16.007 (body P1 / index P1 — agrees).
  - Policy 7 ambiguity on which is source of truth for priority. Downstream consumers (planner, verification coverage) make conflicting assumptions.
- **Proposed Fix:** Decide source of truth (likely BC-INDEX — established pattern) and update BC-2.16.001 line 70 and BC-2.16.009 line 88 to P0.

---

#### P3P27-A-M-002 — `E-CONFIRM-001` (removed code) appears ONLY in test-vectors.md line 113

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** test-vectors.md line 113
- **Policy violated:** 4 (`semantic_anchoring_integrity`)
- **Novelty:** NEW (partially subsumed by C-002; tracked separately for pattern visibility)
- **Description:** test-vectors.md is the sole file in the corpus referencing an error code that error-taxonomy explicitly tombstones. Tracked separately from C-002 because the pattern (supplement emitting removed error codes) warrants a lint rule, not just a one-off fix.
- **Evidence:** error-taxonomy.md line 270: `~~E-CONFIRM-001~~ is removed — its cases are covered by E-FLAG-003 and E-FLAG-008 per DEC-009 and DEC-016.` Grep across specs: `E-CONFIRM-001` appears ONLY in test-vectors.md line 113. Test fixtures would fail immediately; worse, test-writer might resurrect the decommissioned code.
- **Proposed Fix:** Captured in C-002 remediation. Consider adding supplement-wide lint rejecting TVs that reference strikethrough error codes.

---

### LOW

#### P3P27-A-L-001 — `[SCOPE EXPANSION — Phase 3 patch]` markers remain in 5 stories (variant of pass-26 L-001)

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** S-4.06 line 183, S-4.03 line 156, S-1.14, S-1.15, S-4.08
- **Policy violated:** 4 (`semantic_anchoring_integrity` — scope annotation hygiene)
- **Novelty:** NEW (pass-26 L-001 scope was literal `[PHASE 3 PATCH]`; this variant was not covered)
- **Description:** Literal `[PHASE 3 PATCH]` was removed (pass-26 H-001/L-001 closed). But `[SCOPE EXPANSION — Phase 3 patch]` remains in multiple stories. STORY-INDEX §Scope Expansions (lines 419-433) already captures this information permanently. Not convergence-blocking.
- **Evidence:** Grep `\[PHASE 3 PATCH` returns zero hits. Grep `SCOPE EXPANSION` returns hits in S-4.06 line 183, S-4.03 line 156, and others.
- **Proposed Fix:** Post-Phase-2-Patch convergence, strip markers; STORY-INDEX Scope Expansions section remains durable record. Defer if not convergence-blocking.

---

## Observations

- **Invariant citation coverage now strong.** All 32 active DIs have at least one BC citation in `| L2 Invariants |`. Spot-checked DI-001, DI-002, DI-008, DI-012, DI-014, DI-017, DI-018, DI-019, DI-020, DI-022, DI-023, DI-024, DI-026, DI-032 in addition to Burst 27's seven.
- **VP-INDEX arithmetic coherent.** Kani 20 + Proptest 11 + Fuzz 6 + Integration 2 = 39. VP-033/VP-036 correctly re-anchored to S-6.07. VP-039 correctly anchored to S-5.10.
- **BC-INDEX Subsystem Summary arithmetic correct.** BC=195, P0=166, P1=29, Removed=6, Retired=2.
- **Wave BC counts coherent.** Wave 0+1+2+3+4+5+6 raw = 0+69+30+28+45+47+15 = 234 matches line 68.
- **TV-008's BC-2.10.006 stdio scope disclaimer correct.** Body lines 30-38 confirm stdio does NOT govern log forwarding or trust metadata.
- **TV-005's 100K/10MB claim matches BC-2.13.014.** Three-way consistency: TV-005 ↔ S-4.03 Task 8a + AC-9 ↔ BC-2.13.014 body. M-005 closure is solid.
- **ARCH-INDEX Subsystem Registry aligned with BC-INDEX subsystem naming** — verified for sampled subsystems.

## Novelty Assessment

**Novelty: HIGH.** Six of nine findings (C-001, C-002, H-002, H-003, M-001, M-002) originate in the newly-created test-vectors.md supplement — pass-26 could not have flagged them. Two (H-001, H-004) are Wave-1 and Wave-2 drift that Burst 27's sweep list covered or neighbored but did not normalize. One (L-001) is a semantic-marker variant of a prior finding.

These are real convergence blockers: the supplement was supposed to be "binding reference for test-writer agents" but 2 of 10 TVs (TV-002, TV-006) would cause test-writers to implement wrong behavior against real BCs. NOT fine-polish territory — implementation harm is concrete and file:line traceable.

Trajectory: 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 → 3 → 14 → 15 → **9**. CRIT ≠ 0 breaking a 16-pass streak — novel category (supplement defects) introduces critical class.

## Convergence Recommendation

**BLOCK. convergence_counter: 0 of 3.** Two CRITICAL and four HIGH findings. TV-002 and TV-006 must be rewritten before supplement can serve as binding test-writer reference.

**Suggested Burst 28 scope:**
1. Rewrite TV-002 and TV-006 against BC-2.04.009 and BC-2.14.002 source-of-truth (C-001, C-002, M-002).
2. Normalize S-1.14 and S-1.15 body BC tables to canonical schema (H-001).
3. Fix TV-010 traceability row split (H-002).
4. Fix S-1.09 Task 5 and AC-3: `E-FLAG-002` → `E-FLAG-003` (H-003).
5. Fix S-2.01 line 46 BC-2.15.002 title (H-004).
6. Reconcile BC-2.16.001/.009 Priority body vs index (M-001).
7. Defer L-001 to post-convergence hygiene pass.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 4 |
| MEDIUM | 2 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (convergence_counter: 0 of 3)
**Readiness:** requires revision — TV-002 and TV-006 rewrite mandatory before supplement is usable as test-writer reference

## Relevant Files

- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` — TV-002 (lines 88-115), TV-006 (lines 211-234), TV-010 traceability row (line 378)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.04.009-*.md` — source of truth for token TTL/error codes
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.14.002-case-state-transitions.md` — source of truth for case states
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.13.014-*.md` — IOC source (TV-005 matches)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.16.001-*.md` — body line 68 DI-008, DI-030; line 70 P1 vs BC-INDEX P0
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.15.002-*.md` — canonical title source
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-INDEX.md` — v4.9
- `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/VP-INDEX.md` — v1.3, 39 VPs
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/error-taxonomy.md` — E-CONFIRM-001 removed (line 270), E-FLAG-002/003 distinction (lines 70-71)
- `/Users/jmagady/Dev/prism/.factory/specs/domain-spec/invariants.md` — DI-007, DI-025 source
- `/Users/jmagady/Dev/prism/.factory/specs/prd.md` — frontmatter line 12 includes test-vectors.md
- `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` — v1.19, Burst 27 log
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.14-infusion-specs.md` — lines 48-54 non-canonical BC table
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.15-wasm-runtime.md` — lines 48-55 non-canonical BC table
- `/Users/jmagady/Dev/prism/.factory/stories/S-1.09-confirmation-tokens.md` — lines 51, 62 wrong error code
- `/Users/jmagady/Dev/prism/.factory/stories/S-2.01-rocksdb-init.md` — line 46 BC-2.15.002 title drift
- `/Users/jmagady/Dev/prism/.factory/stories/S-4.03-detection-rules.md` — Task 8a + AC-9 reconciled (verified)
- `/Users/jmagady/Dev/prism/.factory/stories/S-4.06-case-management.md` — lines 87-93 correct 5-state machine
- `/Users/jmagady/Dev/prism/.factory/stories/S-4.07-case-metrics.md` — BC-2.14.008/.010/.012 titles canonical (verified)
- `/Users/jmagady/Dev/prism/.factory/stories/S-4.08-action-delivery.md` — 3-col schema verified
