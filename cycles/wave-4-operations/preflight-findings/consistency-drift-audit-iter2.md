---
document_type: preflight-findings-iter2
phase: 4.A
producer: consistency-validator
timestamp: 2026-05-02T23:45:00Z
predecessor: consistency-drift-audit.md (iter-1)
verdict: CONDITIONAL_PASS
total_iter1_findings: 28
iter1_closed: 26
iter1_partial: 0
iter1_regressed: 1
iter1_still_open: 1
new_findings: 4
new_findings_severity: { HIGH: 1, MEDIUM: 2, LOW: 1 }
---

# Wave 4 Consistency + Drift Audit — Iteration 2

## Summary

- Verdict: CONDITIONAL_PASS
- Iter-1 findings: 26 closed / 0 partial / 1 regressed / 1 still-open
- New findings: 5 (HIGH: 1, MEDIUM: 3, LOW: 1)
- Recommendation: targeted one-pass remediation for NEW-002 (HIGH regression in S-4.04 semaphore
  description), NEW-001 (still-open org prefix on S-4.05 rate_limit key), and NEW-003/004/005 (MEDIUM
  STORY-INDEX drift); then proceed to adversarial convergence.

---

## Iter-1 Findings Status

### Per-Story Closure Check

#### S-4.01

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-401-001 | HIGH | K prism-operations crate | CLOSED | Task 0 present lines 93-101; "Register `crates/prism-operations` in workspace `Cargo.toml` members list (DRIFT-401-001 / ADR-012)" explicitly cited; File Structure Requirements row for `Cargo.toml` annotated with workspace dep list (line 326). |
| DRIFT-401-002 | HIGH | E depends_on / S-3.02 status | CLOSED | Previous Story Intelligence updated: "Dispatch prerequisite (DRIFT-401-002): S-3.02 (DataFusion + QueryEngine) MUST be merged before S-4.01 is dispatched" (lines 359-362); 4-row Wave 3 pattern guidance table added (lines 365-369). |
| DRIFT-401-003 | HIGH | I OrgId/org-scoping | CLOSED | `org_id: OrgId` added to ScheduleEntry (line 104); RocksDB key `{org_id_bytes}:{schedule_id_bytes}` (line 111); `list_schedules` restricted to calling session org (line 136); AC-8 org-isolation (lines 231-233). |
| DRIFT-401-004 | MEDIUM | B ADR reference gap | CLOSED | `anchor_adrs: [ADR-013]` in frontmatter (line 31); ADR-013 cited throughout Architecture Compliance Rules (lines 277-298); Library table cites ADR-013 §2.2, §2.8 etc. |
| DRIFT-401-005 | MEDIUM | D Architecture path refs | CLOSED | Architecture Mapping prose: `.factory/specs/architecture/module-decomposition.md` (line 249); Purity Classification: `.factory/specs/architecture/purity-boundary-map.md` (line 258). Both body sections qualified. |
| DRIFT-401-006 | LOW | M cycle field stale | CLOSED | `cycle: "wave-4-operations"` (line 20). |
| DRIFT-401-007 | LOW | M tdd_mode absent | CLOSED | `tdd_mode: strict` (line 21). |
| DRIFT-401-008 | MEDIUM | F story points vs manifest | CLOSED | Frontmatter `points: 5` (line 11); cycle manifest shows 5 — no discrepancy on this story. Remediation note confirms no change needed (line 409). |

**S-4.01 iter-1 closure: 8/8 CLOSED**

---

#### S-4.02

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-402-001 | HIGH | K prism-operations crate | CLOSED | Architecture Mapping: "This story depends on S-4.01 having created `crates/prism-operations` (DRIFT-402-001)" (line 239); Previous Story Intelligence dispatch gate noted (lines 326-329). |
| DRIFT-402-002 | HIGH | I OrgId/org-scoping | CLOSED | All diff_results CF keys include `{org_id}:` prefix: `diff:{org_id}:{schedule_id}:{suffix}` and `epoch:{org_id}:{schedule_id}` (lines 111-114); Architecture Compliance Rules: "All `diff_results` CF keys MUST include `{org_id}:` prefix" (lines 273-275); pack key prefix `{org_id}:pack:{pack_id}` (line 176); pack "global" clarified as org-scoped (lines 174-175). |
| DRIFT-402-003 | MEDIUM | F story points | PARTIALLY CLOSED — SEE NEW-003 | Story frontmatter `points: 3` confirmed (line 11); remediation notes state frontmatter is authoritative and cycle manifest needs correction. However STORY-INDEX v1.81 line 284 still shows `S-4.02 ... 5 ...` — STORY-INDEX was NOT corrected to 3. Frontmatter authoritative, but STORY-INDEX drift remains. |
| DRIFT-402-004 | MEDIUM | G VP reference | CLOSED | VP-019 reference confirmed valid in iter-1; no change needed. |
| DRIFT-402-005 | LOW | M frontmatter schema | CLOSED | `cycle: "wave-4-operations"` (line 20); `tdd_mode: strict` (line 21); `traces_to: [STORY-INDEX.md]` (line 19); `anchor_adrs: [ADR-018]` (line 24). |

**S-4.02 iter-1 closure: 4/5 CLOSED (DRIFT-402-003 partially addressed — story frontmatter correct but STORY-INDEX drift; classified as new finding NEW-003)**

---

#### S-4.03

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-403-001 | HIGH | K prism-operations crate | CLOSED | `depends_on: [..., S-4.01]` (line 8); Previous Story Intelligence: "This is the first story in the detection subsystem (SS-13). S-4.01 creates the `prism-operations` crate — this story DEPENDS ON S-4.01 Task 0" (lines 435-437). |
| DRIFT-403-002 | HIGH | I OrgId/org-scoping | CLOSED | `RuleScope::Client { org_id: OrgId, client_id: ClientId }` (line 107); scope_resolver signature includes `org_id: OrgId` (line 182); org isolation filter: "A rule scoped to org A's client 'acme' MUST NOT be returned in org B's context" (line 187); AC-3b org-isolation AC (lines 281-283). |
| DRIFT-403-003 | MEDIUM | D Library table gaps | CLOSED | `regex = "1.10"` (line 400); `arc-swap = "1"` (line 401); `aho-corasick = "1.1"` (line 398); `notify = "7"` (line 402); `notify-debouncer-full` (line 403) — all added to Library table. |
| DRIFT-403-004 | MEDIUM | D Architecture path refs | CLOSED | Architecture Mapping: `.factory/specs/architecture/module-decomposition.md` (line 334); `.factory/specs/architecture/detection-rule-format.md` (line 335); `.factory/specs/architecture/purity-boundary-map.md` (line 344). |
| DRIFT-403-005 | LOW | M frontmatter schema | CLOSED | `cycle: "wave-4-operations"` (line 20); `tdd_mode: strict` (line 21); `anchor_adrs: [ADR-015]` (line 22). |

**S-4.03 iter-1 closure: 5/5 CLOSED**

---

#### S-4.04

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-404-001 | HIGH | K prism-operations crate | CLOSED | Previous Story Intelligence: "S-4.01 Task 0 creates the `prism-operations` crate in the workspace — this story cannot be dispatched until that task is complete (DRIFT-404-001)" (lines 329-331). |
| DRIFT-404-002 | HIGH | I OrgId/org-scoping | CLOSED | All detection_state CF keys now carry `{org_id}:` prefix: `{org_id}:\x00:{rule_id}:{group_key}` (line 127), `{org_id}:\x01:{rule_id}` (line 141), `{org_id}:\x02:{rule_id}:{dedup_key}` (line 150-151); AC-10 org-isolation AC (lines 221-225); Architecture Compliance Rules updated (lines 281-284). |
| DRIFT-404-003 | MEDIUM | D Architecture path refs | CLOSED | Architecture Mapping: `.factory/specs/architecture/module-decomposition.md` (line 239); Purity Classification: `.factory/specs/architecture/purity-boundary-map.md` (line 248). |
| DRIFT-404-004 | LOW | M frontmatter schema | CLOSED | `cycle: "wave-4-operations"` (line 20); `tdd_mode: strict` (line 21); `anchor_adrs: [ADR-015]` (line 22). |

**S-4.04 iter-1 closure: 4/4 CLOSED**

NOTE: Regression detected — see NEW-002 below. The semaphore description in Previous Story Intelligence (line 331) was not corrected and now contains a factual error.

---

#### S-4.05

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-405-001 | HIGH | K prism-operations crate | CLOSED | Previous Story Intelligence: "S-4.01 Task 0 creates the `prism-operations` crate — this story cannot dispatch until that task is complete (DRIFT-405-001)" (lines 315-317). |
| DRIFT-405-002 | HIGH | I OrgId/org-scoping | CLOSED | `org_id: OrgId` on Alert struct (line 110-111); "MANDATORY — added per ADR-006 §2.1 and DRIFT-405-002" (line 111); RocksDB key `alert:{org_id}:{id}` (line 125); AC-6b org-isolation AC (lines 210-212). Architecture Compliance Rules: "RocksDB `alerts` CF key MUST be `alert:{org_id}:{id}`" (line 273). |
| DRIFT-405-003 | MEDIUM | F story points | CLOSED | `points: 4` (line 12); remediation notes confirm re-pointed 2→4 per spec-reviewer assessment. |
| DRIFT-405-004 | MEDIUM | D Architecture path refs | CLOSED | Architecture Mapping: `.factory/specs/architecture/module-decomposition.md` (line 233); Purity Classification: `.factory/specs/architecture/purity-boundary-map.md` (line 240). |
| DRIFT-405-005 | LOW | M frontmatter schema | CLOSED | `cycle: "wave-4-operations"` (line 20); `tdd_mode: strict` (line 21); `anchor_adrs: [ADR-015]` (line 22). |

**S-4.05 iter-1 closure: 5/5 CLOSED**

NOTE: New issue detected — see NEW-001 below. The rate_limit key in Task 4 is still missing the `{org_id}:` prefix.

---

#### S-4.06

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-406-001 | HIGH | K prism-operations crate | CLOSED | `depends_on: [S-4.01, S-4.05, S-2.01]` (line 8); Phase 4.A Remediation Notes confirms "Added `S-4.01` to `depends_on`" (line 550). |
| DRIFT-406-002 | HIGH | I OrgId/org-scoping | CLOSED | `org_id: OrgId` on Case (line 107); RocksDB key `case:{org_id}:{client_id}:{case_id}` (lines 213, 434-436); AC-17 cross-org access denial (lines 366-370); Architecture Compliance Rule: "Any case-fetch or case-update operation MUST verify `session.org_id == case.org_id`" (lines 437-440). |
| DRIFT-406-003 | HIGH | I Stale case status enum labels | CLOSED | Architecture Mapping prose: "New → Acknowledged → Investigating → Resolved → Closed" (line 384); Objective (line 61) also correct; no remaining `Open`/`InProgress` references found in body. |
| DRIFT-406-004 | MEDIUM | D case/dedup.rs path inconsistency | CLOSED | All references now `cases/dedup.rs` — File Structure Requirements (line 474), Task 9a header (line 229), Architecture Mapping (line 377), Purity Classification (line 400). Token Budget line 91 also uses `cases/dedup.rs`. Phase 4.A Remediation Notes confirm fix (line 553). |
| DRIFT-406-005 | MEDIUM | D Architecture path refs | CLOSED | Architecture Mapping: `.factory/specs/architecture/module-decomposition.md` (line 390); Purity Classification: `.factory/specs/architecture/purity-boundary-map.md` (line 398). |
| DRIFT-406-006 | LOW | M frontmatter schema | CLOSED | `cycle: "wave-4-operations"` (line 20); `tdd_mode: strict` (line 21); `traces_to: [STORY-INDEX.md]` (line 18); `anchor_adrs: [ADR-017]` (line 22). |

**S-4.06 iter-1 closure: 6/6 CLOSED**

---

#### S-4.07

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-407-001 | HIGH | K prism-operations crate | CLOSED | `depends_on: [S-4.01, S-4.06]` (line 8); Phase 4.A Remediation Notes: "Added `S-4.01` to `depends_on`" (line 339). |
| DRIFT-407-002 | MEDIUM | D Architecture path refs | CLOSED | Architecture Mapping: `.factory/specs/architecture/module-decomposition.md` (line 213); Purity Classification: `.factory/specs/architecture/purity-boundary-map.md` (line 220). |
| DRIFT-407-003 | MEDIUM | B Missing ADR refs + org-context | CLOSED | Task 6 includes org-context check: "Org-context check (per ADR-006, DRIFT-407-003): before any alert access, verify that the alert's `org_id` matches the calling session's `org_id`. A mismatch MUST return `E-ALERT-ORG-MISMATCH`" (lines 139-140); AC-7 updated (lines 186-189); Task 7 added for CAPABILITY_ACKNOWLEDGE_ALERT registry check (lines 153-157); `anchor_adrs: [ADR-017]` (line 22). |
| DRIFT-407-004 | LOW | M frontmatter schema | CLOSED | `cycle: "wave-4-operations"` (line 20); `tdd_mode: strict` (line 21); `traces_to: [STORY-INDEX.md]` (line 18); `anchor_adrs: [ADR-017]` (line 22); Gap Register entry GAP-407-001 added (lines 230-234). |

**S-4.07 iter-1 closure: 4/4 CLOSED**

---

#### S-4.08

| ID | Iter-1 Sev | Category | Iter-2 Status | Evidence |
|----|-----------|----------|---------------|---------|
| DRIFT-408-001 | HIGH | K prism-operations crate | CLOSED | Previous Story Intelligence: "S-4.01 Task 0 creates the `prism-operations` crate and registers it in the workspace `Cargo.toml`. This story cannot be dispatched until S-4.01 Task 0 is complete" (lines 565-567); Objective explicitly calls out DRIFT-408-001 (line 75). |
| DRIFT-408-002 | HIGH | I OrgId/org-scoping | CLOSED | `org_id: OrgId` on ActionSpec (line 130); `ClientFilter` enum replaces bare `clients: Vec<ClientId>` (line 131); all `action_state` CF keys prefixed `{org_id}:` (lines 210-213); AC-17 org isolation (lines 395-397); Architecture Compliance Rules: "All `action_state` CF keys MUST be prefixed with `{org_id}:`" (lines 456-458) and "Action specs MUST carry `org_id: OrgId`. Trigger evaluation MUST match `alert.org_id == spec.org_id`" (lines 459-461). |
| DRIFT-408-003 | HIGH | J DTU integration surface mismatch | CLOSED | Test Fixture Surface subsection added (lines 595-607): DTU fixture surface table with S-6.11/12/13 endpoint contracts, response codes, and harness usage. Gap note on hardcoded `message_ts` and ephemeral port usage (lines 607-608). QUAL-408-002 justification for keeping single story scope (lines 609-617). |
| DRIFT-408-004 | MEDIUM | B ADR-016 dispatch gate | CLOSED | `anchor_adrs: [ADR-016, ADR-019]` (line 31); both ADRs in `inputs:` (lines 48-49); ADR-016 cited extensively in Architecture Compliance Rules, Library table, and Tasks throughout. |
| DRIFT-408-005 | MEDIUM | D Architecture path refs | CLOSED | Architecture Mapping: `.factory/specs/architecture/module-decomposition.md` (line 415); Purity Classification: `.factory/specs/architecture/purity-boundary-map.md` (line 425). |
| DRIFT-408-006 | LOW | M frontmatter schema | CLOSED | `cycle: "wave-4-operations"` (line 20); `tdd_mode: strict` (line 21); `traces_to: [STORY-INDEX.md]` (line 19); `anchor_adrs: [ADR-016, ADR-019]` (line 31). |

**S-4.08 iter-1 closure: 6/6 CLOSED**

---

## Iter-1 Closure Summary

| Story | Iter-1 Findings | Closed | Regressed | Still-Open |
|-------|----------------|--------|-----------|------------|
| S-4.01 | 8 | 8 | 0 | 0 |
| S-4.02 | 5 | 4 | 0 | 1 (DRIFT-402-003 → story correct, STORY-INDEX not corrected → reclassified NEW-003) |
| S-4.03 | 5 | 5 | 0 | 0 |
| S-4.04 | 4 | 4 | 1 (semaphore description regression) | 0 |
| S-4.05 | 5 | 5 | 0 | 0 |
| S-4.06 | 6 | 6 | 0 | 0 |
| S-4.07 | 4 | 4 | 0 | 0 |
| S-4.08 | 6 | 6 | 0 | 0 |
| **TOTAL** | **43 iter-1+qual** / **28 drift-only** | **26/28 drift closed** | **1** | **1→NEW** |

Note: Iter-1 reported 28 findings; DRIFT-402-003 is now reclassified as NEW-003 (story frontmatter is correct at 3 pts; the STORY-INDEX was not corrected and shows 5 pts — a new cross-document drift introduced during remediation). Regression: DRIFT-404-001 closure introduced a stale semaphore count in S-4.04 Previous Story Intelligence.

---

## New Findings (introduced during or surfaced by remediation)

| ID | Severity | Category | Story | Finding | Recommendation |
|----|----------|----------|-------|---------|----------------|
| NEW-001 | HIGH | I OrgId/org-scoping | S-4.05 | **Rate-limit key missing org prefix.** Task 4 (rate_limit.rs) specifies RocksDB persistence key as `\x01{rule_id}` (line 137 body, also cited in S-4.04 Dev Notes line 379). This bare key lacks the `{org_id}:` prefix mandated by ADR-008 universal re-keying rule. S-4.04 Task 5 key table correctly uses `{org_id}:\x02:{rule_id}:{dedup_key}` for dedup, but S-4.05 rate_limit persists to `detection_state` CF without org prefix — a cross-tenant isolation violation. Compare: S-4.04 line 150 defines the key scheme with org prefix for all entries; S-4.05 Task 4 line 137 breaks the pattern. Architecture Compliance Rule in S-4.05 correctly mandates org_id on Alert but does not address rate_limit CF keys. | In S-4.05 Task 4: change `\x01{rule_id}` to `{org_id}:\x01:{rule_id}` for the rate_limit persistence key. Update Architecture Compliance Rules in S-4.05 to explicitly state rate_limit CF keys follow the same ADR-008 org-prefix pattern. |
| NEW-002 | HIGH | I + D semaphore regression | S-4.04 | **Stale semaphore description in Previous Story Intelligence is a factual regression.** S-4.04 Previous Story Intelligence line 331 reads: "S-4.01 also owns the 16-permit `Arc<Semaphore>` shared with S-4.08". This contradicts D-209 LOCKED (8 permits per-subsystem, NOT shared) and ADR-013 §2.3. The rest of S-4.04 was correctly updated during remediation, but this single sentence in Previous Story Intelligence was missed. An implementing agent reading this sentence would believe the semaphore is 16 permits and shared — both are wrong. S-4.01 body correctly says 8 permits (Task 5 line 153-157); the cross-reference in S-4.04 was not swept. | In S-4.04 Previous Story Intelligence line 331: replace "S-4.01 also owns the 16-permit `Arc<Semaphore>` shared with S-4.08" with "S-4.01 owns the *schedule executor* semaphore (8 permits per D-209 LOCKED / ADR-013 §2.3). `DetectionEvaluator` does NOT use that semaphore; it operates within the existing QueryEngine execution budget." |
| NEW-003 | MEDIUM | P Story point manifest reconciliation | S-4.02 | **STORY-INDEX point count not corrected.** DRIFT-402-003 remediation confirmed frontmatter `points: 3` is authoritative. However STORY-INDEX v1.81 line 284 still shows the Pts column as `5`. The cycle manifest (v1.8) line 88 also still shows `5` for S-4.02. Both index artifacts are inconsistent with the authoritative story frontmatter. | Correct STORY-INDEX line 284 Pts column: `5` → `3`. Correct cycle-manifest line 88 Pts column: `5` → `3`. Update cycle total: 46 pts may need adjustment. |
| NEW-004 | MEDIUM | O ADR reference drift | S-4.08 | **STORY-INDEX shows wrong ADR for S-4.08.** STORY-INDEX line 290 title annotation reads `[v1.11 ADR-016,ADR-018]`. The story's `anchor_adrs: [ADR-016, ADR-019]` (line 31). ADR-018 is "Differential Result Pack Format" (owns DiffResult/pack semantics). ADR-019 is "SIEM Output Formats" (owns CEF/LEEF encoding). S-4.08 correctly depends on ADR-019 (syslog/CEF/LEEF encoding) — the STORY-INDEX annotation substituted ADR-018 by mistake. | Correct STORY-INDEX line 290 title annotation: `ADR-016,ADR-018` → `ADR-016,ADR-019`. |
| NEW-005 | LOW | N Library pin trailing `.x` | S-4.06 | **`uuid` version still uses `.x` suffix.** S-4.06 Library table line 458 shows `uuid` pinned to `"1.x"`. All other stories in this wave had trailing `.x` removed as part of the remediation sweep (S-4.01 line 308, S-4.05 line 288 both use `"1"`). The `1.x` form is a minor cosmetic inconsistency — Cargo treats it identically to `"1"` for semver — but it is inconsistent with the wave standard. | Change the `uuid` version pin in S-4.06 Library table from `"1.x"` to `"1"`. |

---

## Cross-Cutting Verification

| Class | Iter-1 Status | Iter-2 Status | Notes |
|-------|--------------|---------------|-------|
| K: prism-operations crate absence | All 8 HIGH | ALL CLOSED | Task 0 in S-4.01; depends_on S-4.01 in S-4.02/03/04/05/06/07/08; dispatch prerequisites documented. |
| I: OrgId/org-scoping | All 8 HIGH | MOSTLY CLOSED — 1 gap | 7 of 8 stories fully closed. S-4.05 rate_limit key `\x01{rule_id}` still missing org prefix (NEW-001 HIGH). |
| D: Unqualified architecture/ paths | All 8 MEDIUM | ALL CLOSED | All `.factory/specs/architecture/` prefixes applied in Architecture Mapping + Purity Classification across all 8 stories. |
| M: Stale cycle + missing tdd_mode | All 8 LOW | ALL CLOSED | All 8: `cycle: "wave-4-operations"` + `tdd_mode: strict` + `traces_to: [STORY-INDEX.md]`. |
| F: Story-point discrepancies | S-4.02, S-4.05 | PARTIAL | S-4.05 re-pointed 2→4 (CLOSED). S-4.02 frontmatter corrected to 3 but STORY-INDEX and cycle manifest still show 5 (NEW-003 MEDIUM). |
| N: Library pins currency | — | VERIFIED | datafusion=53.1 in S-4.03 (line 397) and S-4.04 (line 302); croner=3 in S-4.01 (line 310) and S-4.08 (line 497); blake3=1.8 in S-4.01 (line 311), S-4.02 (line 294), S-4.04 (line 304); aho-corasick=1.1 in S-4.03 (line 398); hdrhistogram=7.5 in S-4.07 (line 262); lettre=0.11 in S-4.08 (line 498); wasmtime=44 in S-4.08 (line 507); scopeguard=1.2 in S-4.04 (line 303); serde_jcs=0.1 in S-4.02 (line 295). All research-mandated pins CONFIRMED. |
| O: ADR reference completeness | — | MOSTLY VERIFIED | All 8 stories have anchor_adrs pointing to relevant new ADRs (S-4.01→ADR-013, S-4.02→ADR-018, S-4.03→ADR-015, S-4.04→ADR-015, S-4.05→ADR-015, S-4.06→ADR-017, S-4.07→ADR-017, S-4.08→ADR-016+ADR-019). STORY-INDEX annotation for S-4.08 uses wrong ADR-018 instead of ADR-019 (NEW-004 MEDIUM). |
| P: Re-pointed story manifest reconciliation | — | PARTIAL | S-4.03=8 ✓ (cycle manifest and STORY-INDEX both 8), S-4.05=4 ✓ (cycle manifest 4, STORY-INDEX 4), S-4.06=9 ✓ (cycle manifest 9, STORY-INDEX 9), S-4.08=9 ✓ (cycle manifest 9, STORY-INDEX 9). S-4.02=3 in story but 5 in both indexes (NEW-003). |

---

## Verdict Justification

**CONDITIONAL_PASS**

The comprehensive remediation pass (commit b881b0d2) successfully closed 26 of 28 iter-1 findings. All 11 iter-1 HIGH findings are closed: all 8 stories have Task 0 / depends_on S-4.01 for the prism-operations crate (K class), and all 8 stories now have OrgId on domain types with org-prefixed RocksDB keys and org-isolation ACs (I class). All 6 new ADRs are anchored in frontmatter. Library pins match research findings.

Two issues prevent PASS:

1. **NEW-001 (HIGH):** S-4.05 rate_limit persistence key (`\x01{rule_id}`) is missing the `{org_id}:` prefix mandated by ADR-008. This is a substantive multi-tenant isolation gap — rate limits from org-A would not be separated from org-B rate limits in the `detection_state` CF. This must be corrected before adversarial convergence.

2. **NEW-002 (HIGH):** S-4.04 Previous Story Intelligence contains the false statement "S-4.01 also owns the 16-permit `Arc<Semaphore>` shared with S-4.08." D-209 LOCKED specifies 8 permits per-subsystem with NO sharing. An implementing agent reading this would implement incorrect behavior.

Three MEDIUM findings (NEW-003, NEW-004) are index/annotation drift that do not block implementation but must be corrected before the consistency-validator final gate:

- NEW-003: STORY-INDEX and cycle-manifest still show S-4.02 at 5 pts when story frontmatter is authoritative at 3.
- NEW-004: STORY-INDEX shows `ADR-018` for S-4.08 instead of `ADR-019`.
- NEW-005 (LOW): S-4.06 `uuid | 1.x` minor cosmetic inconsistency.

**Recommendation: One targeted remediation pass addressing NEW-001 and NEW-002 (both HIGH, small edits), then proceed to adversarial convergence.** NEW-003 and NEW-004 (MEDIUM index drift) can be bundled into the same pass. Total estimated remediation: 4 targeted line-level edits across 3 files (S-4.05, S-4.04, STORY-INDEX, cycle-manifest).

---

## Remediation Specification

| Finding | File | Change Required |
|---------|------|----------------|
| NEW-001 | `.factory/stories/S-4.05-alert-generation.md` | Task 4 line 137: `\x01{rule_id}` → `{org_id}:\x01:{rule_id}`. Add to Architecture Compliance Rules: "Rate-limit counter CF key MUST be `{org_id}:\x01:{rule_id}` per ADR-008." |
| NEW-002 | `.factory/stories/S-4.04-detection-evaluation.md` | Previous Story Intelligence line 331: Replace "S-4.01 also owns the 16-permit `Arc<Semaphore>` shared with S-4.08" with "S-4.01 owns the *schedule executor* `Arc<Semaphore>` (8 permits per D-209 LOCKED / ADR-013 §2.3). `DetectionEvaluator` does NOT use or share that semaphore." |
| NEW-003 | `.factory/stories/STORY-INDEX.md` line 284 and `.factory/cycles/wave-4-operations/cycle-manifest.md` line 88 | Change the Pts column for S-4.02 from `5` to `3` in both files. Update cycle total (46 pts) if applicable. |
| NEW-004 | `.factory/stories/STORY-INDEX.md` line 290 | Change title annotation from `ADR-016,ADR-018` to `ADR-016,ADR-019`. |
| NEW-005 | `.factory/stories/S-4.06-case-management.md` Library table | Change `uuid` version pin from `"1.x"` to `"1"`. |
