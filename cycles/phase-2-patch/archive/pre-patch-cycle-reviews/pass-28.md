---
document_type: adversarial-review
level: ops
version: "1.0"
status: findings-open
producer: adversary
timestamp: 2026-04-19T00:00:00
phase: 2
inputs: []
input-hash: "[md5]"
traces_to: prd.md
pass: 28
previous_review: pass-27.md
novelty: MEDIUM
findings: 5
critical: 0
high: 2
medium: 2
low: 1
previous_pass: 27 (9 findings: 2 CRIT, 4 HIGH, 2 MED, 1 LOW — 9 closed Burst 28 + 19 preemptive drift fixes applied)
convergence_counter: 0 of 3
---

# Adversarial Review: Prism (Pass 28)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: Cycle prefix from `.factory/current-cycle` (e.g., `P1CONV`, `P3PATCH`)
  - If no current-cycle file exists, omit the cycle segment (falls back to `ADV-P<PASS>-<SEV>-<SEQ>`)
- `<PASS>`: Two-digit pass number (e.g., `P01`, `P24`)
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples: `ADV-P1CONV-P03-CRIT-001`, `ADV-P3PATCH-P24-HIGH-002`, `ADV-P01-MED-003` (no cycle)

The cycle prefix prevents ID collisions when multiple convergence cycles coexist in the same project.

## Scope

Fresh-context adversarial review at commit `667556d`. Focus: Burst 28 closure verification (9 pass-27 findings + 19 preemptive fixes), test-vectors v2.0 structural audit, independent BC spot-checks (BC-2.01/.03/.08/.17/.18/.19), VP-INDEX arithmetic (Policy 9), Invariant-to-BC orphan scan (Policy 2), Story frontmatter-body coherence (Policy 8), BC-INDEX v4.10 / STORY-INDEX v1.20 integrity.

## Part A — Fix Verification

| Pass-27 Finding | Claim | Verified | Evidence |
|---|---|---|---|
| P3P27-A-C-001 (TV-006 case states + E-CASE-003→004) | Closed | **YES** | test-vectors.md TV-006: canonical `New/Acknowledged/Investigating/Resolved/Closed`; E-CASE-004; FalsePositive correctly scoped as DispositionCode |
| P3P27-A-C-002 (TV-002 15m→5m, UUID-v4→crypto-random, E-CONFIRM-001→E-FLAG-007) | Closed | **YES** | test-vectors.md TV-002: 300s TTL, `<GENERATED:crypto-random>`, `E-FLAG-007` for cap |
| P3P27-A-H-001 (S-1.14/S-1.15 canonical 2-col BC schema) | Closed | **YES** | Both use `| BC ID | Title |`; titles verbatim from BC-INDEX v4.10 |
| P3P27-A-H-002 (TV-010 DI-030/031 split) | Closed | **YES** | BC-2.16.001 traces DI-008+DI-030; BC-2.16.007 traces DI-030+DI-031 |
| P3P27-A-H-003 (S-1.09 E-FLAG-002→E-FLAG-003 token expiry) | **PARTIAL** | Task 5 + AC-3 fixed; BUT new drift — see H-001 below (cap code + UUID type) |
| P3P27-A-H-004 (S-2.01 BC-2.15.002 title) | Closed | **YES** | `removeRange` + `per Domain` present |
| P3P27-A-M-001 (BC-2.16.001/.009 Priority P0) | Closed | **YES** | Both body Priority = P0 |
| P3P27-A-M-002 (E-CONFIRM-001 sweep) | Closed | **YES** | Only remaining reference is the explicit removal marker in error-taxonomy.md line 270 |
| P3P27-A-L-001 ([SCOPE EXPANSION] markers) | **PARTIAL (expected)** | S-1.14/S-2.01/S-6.01 swept; S-4.03/S-4.06 retain markers per deferral |

**Preemptive drift fixes (19 claimed):** 18 verified correct. 1 incomplete — S-3.04 preemptive fix addressed BC-2.11.009 em-dash but missed 4 adjacent MCP-tool-name backtick drifts (see H-001 below). Also S-2.01 preemptive sweep fixed BC-2.15.002 but missed adjacent BC-2.15.005 "Operation" word (see M-001).

**Wave-6 reference-role schema question resolved:** LEGITIMATE alternate schema — S-6.01/.02/.07 have empty `behavioral_contracts: []` and consume BCs via `depends_on:`. Policy 8 does not apply. No drift.

**Policy 8 S-3.07 question resolved:** Genuine minor gap — BC-2.04.005 in frontmatter without AC trace. See L-001.

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

None.

### HIGH

#### P3P28-A-H-001 — S-1.09 Notes drifts from BC-2.04.009 on token_id type and cap error code

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-1.09-confirmation-tokens.md` lines 49, 148, 149
- **Description:** Pass-27 H-003 only addressed the expiry error code in S-1.09; cap-code + token-type drift in the story body was not back-propagated from the Burst 28 test-vectors.md v2.0 fix. Implementer reading S-1.09 will pin `uuid::Uuid::now_v7()` and return `E-FLAG-003` for cap; test-writer reading TV-002 will assert `<GENERATED:crypto-random>` and `E-FLAG-007`. The two artifacts cannot both be satisfied.
- **Evidence:**
  - Line 49 (Task 3): `"generate token with UUID v7, SHA-256 content hash..."` — BC-2.04.009 line 34 says `cryptographic random string` (not UUID v7)
  - Line 148 (Notes): `"UUID v7 is used for tokens because it embeds a timestamp..."` — justifies UUID v7 but BC does not sanction this
  - Line 149 (Notes): `"When the cap is hit, the system MUST return E-FLAG-003 (token cap exceeded)"` — contradicts BC-2.04.009 lines 42+55 which assign `E-FLAG-007` to cap-exceeded; `E-FLAG-003` is reserved for expired tokens
- **Proposed Fix:** S-1.09 Task 3 + Notes lines 148-149 — replace UUID-v7 with `cryptographic random string` per BC; fix cap code E-FLAG-003 → E-FLAG-007; clarify E-FLAG-003 covers expiry.

**Policy violated:** 7 (bc_h1_is_title_source_of_truth), 4 (semantic_anchoring_integrity), 8 (propagation)
**Confidence:** HIGH
**Novelty:** NEW — pass-27 H-003 only addressed expiry code; cap-code + token-type drift in S-1.09 body was not in scope

---

#### P3P28-A-H-002 — S-3.04 body BC table lacks backticks on 4 MCP tool titles

- **Severity:** HIGH (4 BC title drifts in one story = pattern threshold)
- **Category:** spec-fidelity
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-3.04-alias-system.md` lines 47, 49, 50, 51
- **Description:** S-3.03 correctly quoted `` `explain_query` ``; S-3.04's 4 alias-MCP-tool BCs were missed in the preemptive sweep. Multi-BC title drift in single story. Story narrative at line 239 correctly uses `` `create_alias` `` backticks, so the BC table is inconsistent within its own story.
- **Evidence:**
  - Line 47: `| BC-2.11.008 | create_alias MCP Tool |` — BC-INDEX line 145: `` `create_alias` MCP Tool ``
  - Line 49: `| BC-2.11.013 | list_aliases MCP Tool |` — BC-INDEX line 150: `` `list_aliases` MCP Tool ``
  - Line 50: `| BC-2.11.014 | delete_alias MCP Tool |` — BC-INDEX line 151: `` `delete_alias` MCP Tool ``
  - Line 51: `| BC-2.11.015 | explain_alias MCP Tool |` — BC-INDEX line 152: `` `explain_alias` MCP Tool ``
- **Proposed Fix:** Add backticks around 4 tool names to match BC-INDEX verbatim.

**Policy violated:** 7 (bc_h1_is_title_source_of_truth, exact match)
**Confidence:** HIGH
**Novelty:** NEW

### MEDIUM

#### P3P28-A-M-001 — S-2.01 BC-2.15.005 title missing word "Operation"

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-2.01-rocksdb-init.md` line 47
- **Description:** Preemptive sweep fixed BC-2.15.002 but missed the adjacent BC-2.15.005 entry. Single-BC word-level drift breaks literal-match checks.
- **Evidence:**
  - Line 47: `| BC-2.15.005 | Crash Recovery Dirty Bits — Set Before, Clear After, Detect on Restart |`
  - BC-INDEX line 195: `| BC-2.15.005 | Crash Recovery Dirty Bits — Set Before Operation, Clear After, Detect on Restart |`
- **Proposed Fix:** Insert "Operation" → `Set Before Operation, Clear After, Detect on Restart`.

**Policy violated:** 7 (bc_h1_is_title_source_of_truth, verbatim)
**Confidence:** HIGH
**Novelty:** NEW

---

#### P3P28-A-M-002 — test-vectors.md mis-attributes VP-034 to BC-2.05.003 (Policy 4 semantic anchoring; survived 27 passes)

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` lines 49, 51, 297
- **Description:** VP-034 verifies AES-GCM round-trip on `prism-credentials` — a completely unrelated property to audit-redaction on `prism-audit`. Surface keyword "credential" fooled every prior pass. Test-writer following this trace into prism-audit will find no round-trip property to verify; may fabricate a redaction proptest and mis-label it VP-040, polluting the VP catalog.
- **Evidence:**
  - Line 49 (TV-001 table row): `| ... | ... | invariant | VP-034 anchor; DI-002 enforced |`
  - Line 51 (Trace): `**Trace:** BC-2.05.003 postconditions 1-4, VP-034, DI-002`
  - Line 297 (Traceability Matrix): `| TV-001 | SS-05 | BC-2.05.003 | DI-002 | VP-034 |`
  - VP-INDEX.md line 55: `| VP-034 | Encryption round-trip: encrypt then decrypt returns plaintext | prism-credentials | proptest | P0 | draft | S-1.06 |`
  - BC-2.05.003 file traceability (lines 49-54): NO VP anchor — only `L2 Invariants: DI-002` and `Priority: P0`
- **Proposed Fix:** At lines 49, 51, 297 — remove VP-034 citation, replace with "integration only". Do NOT create spurious VP-040.

**Policy violated:** 4 (semantic_anchoring_integrity), 9 (vp_index_is_vp_catalog_source_of_truth)
**Confidence:** HIGH
**Novelty:** NEW — surfaced during pass-28 traceability-matrix deep-read; survived all 27 prior passes

### LOW

#### P3P28-A-L-001 — S-3.07 frontmatter contains BC-2.04.005 with no AC trace (Policy 8)

- **Severity:** LOW (single-BC minor; story-writer flagged in Burst 28)
- **Category:** spec-fidelity
- **Location:** `/Users/jmagady/Dev/prism/.factory/stories/S-3.07-write-execution.md` lines 20, 285-318
- **Description:** BC-2.04.005 (Hidden Tools Pattern) materially affects story scope (write verbs hidden when capability denied — Task 3 line 155). Without AC trace, no test vehicle.
- **Evidence:**
  - Frontmatter line 20: `behavioral_contracts: [BC-2.04.001, BC-2.04.005, BC-2.04.007, BC-2.04.008, BC-2.05.009]`
  - Body BC table line 48 includes BC-2.04.005
  - ACs (lines 285-318) trace BC-2.04.007/008 (AC-1), BC-2.04.008 (AC-2), BC-2.05.009 (AC-5), BC-2.04.007 (AC-6), BC-2.04.001/BC-2.05.009 (AC-7). Zero AC traces to BC-2.04.005.
- **Proposed Fix:** Add AC-9: `Given a write verb for a sensor whose write capability is denied, When PrismQL parsing runs, Then the verb is rejected with E-FLAG-001. (BC-2.04.005)`.

**Policy violated:** 8 (bc_array_changes_propagate_to_body_and_acs)
**Confidence:** HIGH
**Novelty:** RE-CONFIRMED

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 2 |
| MEDIUM | 2 |
| LOW | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate (Burst 29 required)
**Readiness:** requires revision before Phase 3

---

## Observations

1. **STORY-INDEX BC-INDEX version pin stale.** STORY-INDEX v1.20 lines 24 and 71 pin `BC-INDEX.md v4.8`, actual is v4.10. Not a correctness failure (counts unchanged) but a Policy 3 state-manager signal. Non-blocking.

2. **S-1.09 contains heavy `[TODO]` template placeholders** — inherited from template scaffolding, explicitly acceptable in draft-status stories.

3. **TV-008 correctly uses "integration only"** (line 304) — the exact pattern M-002 should apply to TV-001.

4. **Independent BC spot-check passed** for BC-2.01.002, BC-2.03.001, BC-2.17.001, BC-2.18.*. H1 titles match BC-INDEX; priorities consistent; subsystem frontmatter aligned with ARCH-INDEX. No fresh drift.

5. **Policy 2 DI orphan scan clean.** All 32 active DIs have at least one BC citation; removed DIs properly tombstoned.

6. **Policy 9 arithmetic clean.** VP-INDEX 39 rows = 20 Kani + 11 Proptest + 6 Fuzz + 2 Integration. Coverage matrix module sums match.

7. **Test-vectors v2.0 template conformance PASSES** — frontmatter, required sections, table column schema, Category taxonomy, Traceability Matrix all conform. Only substantive finding is M-002 (VP mis-citation), not a structural defect.

---

## Novelty Assessment

**NOVELTY: MEDIUM.**

All 5 findings are NEW — not restatements of pass-27. Three correlate to the same root cause: **preemptive drift sweep was targeted (specific BC titles / specific stories) rather than comprehensive (re-derive all BC-table rows from BC-INDEX)**. An index-wide 2-col-schema verification pass would catch H-002, M-001, and similar future drift in one sweep.

Key novelty class: **H-001 exposes "drift moves, does not disappear"** — Burst 28's test-vectors.md v2.0 fix for C-002 corrected token-id type and cap error code in the test vectors, but did not propagate corrections back to S-1.09 story body. Fresh context specifically designed to detect this asymmetry.

Key novelty class: **M-002 exposes 27-pass-old Policy 4 violation** — VP-034 (encrypt round-trip, prism-credentials) mis-cited for BC-2.05.003 (audit redaction, prism-audit). Surface keyword ("credential") fooled every prior pass including the fresh Burst 28 rewrite.

Trajectory: 26→8→4→2→1→1→3→6→12→8→6→7→3→14→15→9→**5**. Clear downward trend. CRIT=0 restored.

---

## Convergence Recommendation

**BLOCK convergence at 0/3.** 5 findings:
- 2 HIGH (H-001, H-002) are Policy 7/4/8 violations that would mislead implementers
- 2 MEDIUM (M-001, M-002) are propagation drift that should resolve before Phase 3
- 1 LOW (L-001) is already-flagged and trivial

Recommended Burst 29 scope (all surgical, no architectural change):
1. S-1.09 back-propagate BC-2.04.009 (H-001): Task 3 + Notes 148-149 — UUID-v7 → crypto-random; cap E-FLAG-003 → E-FLAG-007
2. S-3.04 backtick sweep (H-002): 4 tool names on lines 47/49/50/51
3. S-2.01 Operation-word (M-001): line 47 BC-2.15.005
4. test-vectors.md VP-034 de-mis-citation (M-002): lines 49, 51, 297 — replace with "integration only"
5. S-3.07 AC-9 addition (L-001): 3-line Given/When/Then for BC-2.04.005
6. (Optional) STORY-INDEX v4.8 → v4.10 pin bump

Expected post-Burst-29: 0 findings → counter 1/3. 2 more clean passes needed for full Phase 2 Patch convergence.

---

## Relevant Files

- `/Users/jmagady/Dev/prism/.factory/stories/S-1.09-confirmation-tokens.md` — H-001 (lines 49, 148, 149)
- `/Users/jmagady/Dev/prism/.factory/stories/S-3.04-alias-system.md` — H-002 (lines 47, 49, 50, 51)
- `/Users/jmagady/Dev/prism/.factory/stories/S-2.01-rocksdb-init.md` — M-001 (line 47)
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/test-vectors.md` — M-002 (lines 49, 51, 297)
- `/Users/jmagady/Dev/prism/.factory/stories/S-3.07-write-execution.md` — L-001 (add AC-9 around line 318)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.04.009-confirmation-token-request.md` — H-001 SoT (lines 34, 42, 55)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-INDEX.md` — H-002/M-001 SoT (lines 145, 150, 151, 152, 195)
- `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/VP-INDEX.md` — M-002 SoT (line 55)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.05.003-secret-redaction-in-audit-entries.md` — M-002 confirms no VP anchored
- `/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/error-taxonomy.md` — H-001 evidence (line 270 E-CONFIRM-001 removed; E-FLAG-003 vs E-FLAG-007)
- `/Users/jmagady/Dev/prism/.factory/stories/STORY-INDEX.md` — Observation 1 (lines 24, 71 stale v4.8 pin)
