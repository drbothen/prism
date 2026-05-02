---
document_type: preflight-findings
phase: 4.A
producer: consistency-validator
timestamp: 2026-05-02T18:00:00Z
inputs:
  - .factory/stories/S-4.01-schedule-crud.md
  - .factory/stories/S-4.02-diff-results-packs.md
  - .factory/stories/S-4.03-detection-rules.md
  - .factory/stories/S-4.04-detection-evaluation.md
  - .factory/stories/S-4.05-alert-generation.md
  - .factory/stories/S-4.06-case-management.md
  - .factory/stories/S-4.07-case-metrics.md
  - .factory/stories/S-4.08-action-delivery.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md (v4.27)
  - .factory/stories/STORY-INDEX.md (v1.80)
  - .factory/specs/verification-properties/VP-INDEX.md (v1.19)
  - .factory/specs/domain-spec/capabilities.md (v1.14)
  - .factory/specs/architecture/decisions/ (ADR-001 through ADR-014)
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/concurrency-architecture.md
  - .factory/specs/architecture/actions.md (AD-021)
  - .factory/specs/architecture/module-decomposition.md
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/architect-adr-identification.md
  - Cargo.toml (workspace)
  - crates/prism-storage/Cargo.toml
  - git log --oneline -20 develop
verdict: FAIL
total_findings: 28
severity_breakdown: { HIGH: 11, MEDIUM: 12, LOW: 5 }
---

# Wave 4 Consistency + Drift Audit Findings

## Summary

- Stories audited: 8
- Total drift items: 28 (HIGH: 11, MEDIUM: 12, LOW: 5)
- Cross-cutting drift classes: K (prism-operations crate does not exist), I (org-scoping absent from all 8 stories), D (path-unqualified architecture refs), M (frontmatter cycle/tdd_mode schema), F (story-point discrepancies between stories and cycle manifest)
- Verdict: FAIL

All 8 stories are **blocked from dispatch** until HIGH findings are remediated. The most systemic HIGH items are: (K) every story assumes `prism-operations` already exists in the workspace — it does not; (I) zero stories establish OrgId/OrgSlug scoping for schedules, rules, cases, alerts, or actions despite Wave 3's mandatory multi-tenant architecture (ADR-006); and (D) the key dependency story S-3.02 is still `status: draft`, not merged.

---

## Per-Story Findings

### S-4.01 Schedule CRUD and Execution Loop

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-401-001 | HIGH | K prism-operations crate | `target_module: prism-operations` and all File Structure Requirements reference `crates/prism-operations/` (line 92), but `prism-operations` is absent from the workspace `Cargo.toml` members list and absent from `crates/` directory. The crate must be created as part of Wave 4, but the story body contains no "Create Cargo.toml and add to workspace" step and no workspace registration instruction. | Add Task 0: "Register `crates/prism-operations` in workspace `Cargo.toml` members." Ensure File Structure Requirements row for `crates/prism-operations/Cargo.toml` is annotated with workspace-level workspace deps to add. |
| DRIFT-401-002 | HIGH | E depends_on / D dep status | `depends_on: [S-3.02, S-2.01]` (frontmatter line 8). S-2.01 is MERGED (PR #43). S-3.02 is `status: draft` — not merged, dispatch unblocked state unknown. If S-4.01 proceeds before S-3.02 merges, `QueryEngine::execute_scheduled()` referenced in Task 5 and the `GreedyMemoryPool` sharing pattern referenced in Previous Story Intelligence will not exist. | Gate S-4.01 dispatch behind S-3.02 merge. Story body must be updated after S-3.02 merges to confirm the `SessionContext` sharing API surface. |
| DRIFT-401-003 | HIGH | I OrgId/org-scoping | `ScheduleEntry` struct (Task 1 line 93) defines `clients: Vec<ClientId>` but zero mention of OrgId, OrgSlug, or OrgRegistry scoping. Wave 3 ADR-006 mandates that all operational objects are org-scoped via `OrgId` (UUID v7). The `ClientId` type is a Wave 1/2 alias; per cycle-manifest §Deprecations, `TenantId` alias is scheduled for Wave 4 removal (D-157). There is no Wave 3 OrgId guard on schedule creation, listing, or deletion. No AC verifies org isolation for schedules. | Add `org_id: OrgId` to `ScheduleEntry`. Add an AC: "Given schedules belonging to org A, When `list_schedules` is called in org B's context, Then org A's schedules are not returned." Reference ADR-006 §2.1. |
| DRIFT-401-004 | MEDIUM | D ADR reference gap | Story body does not cite any Wave 3 ADRs (ADR-006 through ADR-014). These are now ACCEPTED with IMPLEMENTED status. Schedule semantics decision (ADR-013 per D-204 and architect's ADR identification) does not yet exist but should be referenced once authored. | After ADR-013 is authored per D-204, add `anchor_adrs: [ADR-013]` to frontmatter and cite it in Architecture Compliance Rules. |
| DRIFT-401-005 | MEDIUM | D Architecture path refs | Lines 199, 208 reference `architecture/module-decomposition.md` and `architecture/purity-boundary-map.md` without the `.factory/specs/` prefix. The correct canonical path is `.factory/specs/architecture/module-decomposition.md`. Story v1.4 changelog notes that `architecture/` prefix was already fixed in `inputs:` (line 326), but the body prose was not swept. Affects: line 199, 208. Same pattern appears in all other 7 stories. See DRIFT-cross-001 below. | Qualify all in-body `architecture/` refs to `.factory/specs/architecture/`. |
| DRIFT-401-006 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"` (frontmatter line 20). STATE.md `current_cycle` is `wave-3-multi-tenant` (D-205 confirms Wave 4 cycle name is `wave-4-operations`). The `cycle` field is stale. Merged Wave 3 stories use `cycle: "v1.0.0-greenfield"` as a legacy value, but the cycle-manifest establishes `wave-4-operations` as the canonical Wave 4 cycle identifier. | Update `cycle: "wave-4-operations"`. |
| DRIFT-401-007 | LOW | M Frontmatter schema | `tdd_mode` field is absent. Recent merged Wave 3 stories (S-3.3.03, W3-FIX-CODE-002) include `tdd_mode: strict`. All Wave 4 stories are P0 and should inherit `tdd_mode: strict` per VSDD convention. | Add `tdd_mode: strict` to frontmatter. |
| DRIFT-401-008 | MEDIUM | M Story points vs cycle manifest | Frontmatter `points: 5` (line 12). Cycle manifest Wave 4 Story Inventory row for S-4.01 also shows 5 pts. No discrepancy for S-4.01 itself, but the cycle manifest total says 45 points; per-story frontmatter sum yields 33 — an 11-point gap. This is a manifest arithmetic error, not a story error, but noted for completeness. Tracked under DRIFT-manifest-001. | Correct the cycle manifest total point count after per-story points are reconciled. |

### S-4.02 Differential Results and Packs

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-402-001 | HIGH | K prism-operations crate | Same as DRIFT-401-001 — all File Structure Requirements reference `crates/prism-operations/src/diff/` and `crates/prism-operations/src/pack/`, but the crate does not exist. | Same remediation as DRIFT-401-001. |
| DRIFT-402-002 | HIGH | I OrgId/org-scoping | `DiffResult`, `EpochTracker`, and pack expansion have no org-scoping. Packs are described as "global (not client-scoped)" (Task 7, line 142), but in the multi-tenant architecture each pack MUST be scoped to an org. The `diff:{schedule_id}:prev` key (Task 2 line 106) has no OrgId prefix, which violates ADR-008 CF key universals. | Add OrgId to diff CF key prefix per ADR-008. Clarify pack "global" meaning within a single org context vs across orgs. |
| DRIFT-402-003 | MEDIUM | F Story points | Frontmatter `points: 3` (line 10). Cycle manifest Wave 4 Story Inventory row for S-4.02 shows 5 pts. Discrepancy: story vs manifest. | Reconcile — either update manifest or story. |
| DRIFT-402-004 | MEDIUM | G VP reference | `verification_properties: [VP-019]` (line 24). VP-INDEX shows VP-019 anchor story is S-4.02 and method is `proptest`. Consistent. No renumbering drift. Confirmed clean. | No action needed. |
| DRIFT-402-005 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"` — same stale cycle field as DRIFT-401-006. `tdd_mode` absent. | Update cycle to `wave-4-operations`; add `tdd_mode: strict`. |

### S-4.03 Detection Rule Loading and Compilation

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-403-001 | HIGH | K prism-operations crate | Same as DRIFT-401-001. All detection rule file paths reference a non-existent crate. | Same remediation. |
| DRIFT-403-002 | HIGH | I OrgId/org-scoping | `RuleScope` enum (Task 1 line 102) includes `Global`, `Client(ClientId)`, and `Analyst(AnalystId)` but does not include `OrgId` scoping. `Client(ClientId)` should be `Client(OrgId, ClientId)` or mapped through the OrgRegistry per ADR-006 to prevent cross-org rule visibility. No AC verifies that a rule scoped to org A's client "acme" is not returned by `list_rules` in org B's context. | Map `ClientId` through `OrgRegistry` in the `Client` scope variant. Add org-isolation AC. Reference ADR-006. |
| DRIFT-403-003 | MEDIUM | D Library table gaps | The Library & Framework Requirements table (lines 344-355) is missing `regex` and `arc-swap`, both of which are used in body Tasks 8a (IOC file loading) at lines 188-210. `regex::Regex`, `regex::RegexSet` (from `regex` crate) and `arc_swap::ArcSwap` are named in task prose but absent from the dependency table. | Add `regex = "1.x"` and `arc-swap = "1.x"` rows to the Library & Framework Requirements table. |
| DRIFT-403-004 | MEDIUM | D Architecture path refs | Lines 295, 297, 305 use unqualified `architecture/module-decomposition.md`, `architecture/detection-rule-format.md`, `architecture/purity-boundary-map.md`. Same pattern as DRIFT-401-005. | Qualify paths. |
| DRIFT-403-005 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"`; `tdd_mode` absent. | Update cycle; add tdd_mode. |

### S-4.04 Detection Evaluation (Single/Correlation/Sequence)

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-404-001 | HIGH | K prism-operations crate | Same as DRIFT-401-001. | Same remediation. |
| DRIFT-404-002 | HIGH | I OrgId/org-scoping | Detection state CF keys (`\x00{rule_id}:{group_key}`, `\x01{rule_id}`, `\x02{rule_id}:{dedup_key}`) in Task 5 (lines 138-150) carry no OrgId prefix. In a multi-tenant deployment, correlation windows and sequence trackers for org A must be isolated from org B. Missing OrgId in keys violates ADR-008 universal re-keying rule. | Prepend `{org_id}:` to all `detection_state` CF keys, consistent with ADR-008 §2.1. |
| DRIFT-404-003 | MEDIUM | D Architecture path refs | Lines 220, 229 use unqualified `architecture/module-decomposition.md`, `architecture/purity-boundary-map.md`. | Qualify paths. |
| DRIFT-404-004 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"`; `tdd_mode` absent. | Update cycle; add tdd_mode. |

### S-4.05 Alert Generation

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-405-001 | HIGH | K prism-operations crate | Same as DRIFT-401-001. | Same remediation. |
| DRIFT-405-002 | HIGH | I OrgId/org-scoping | `Alert` struct (Task 3 line 108) contains `client_id: ClientId` but no `org_id: OrgId`. Alerts stored under key `alert:{id}` (line 119) have no org prefix, violating ADR-008. The `alerts` CF is shared; without OrgId prefix, `list_alerts` could leak cross-org results. | Add `org_id: OrgId` to `Alert`. Prefix RocksDB key `alert:{org_id}:{id}`. Add org-isolation AC. |
| DRIFT-405-003 | MEDIUM | F Story points | Frontmatter `points: 2` (line 10). Cycle manifest shows S-4.05 at 1 pt. Discrepancy. | Reconcile. |
| DRIFT-405-004 | MEDIUM | D Architecture path refs | Lines 190, 199 use unqualified `architecture/module-decomposition.md`, `architecture/purity-boundary-map.md`. | Qualify paths. |
| DRIFT-405-005 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"`; `tdd_mode` absent. | Update cycle; add tdd_mode. |

### S-4.06 Case Management

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-406-001 | HIGH | K prism-operations crate | Same as DRIFT-401-001. | Same remediation. |
| DRIFT-406-002 | HIGH | I OrgId/org-scoping | `Case` struct (Task 1 line 105) includes `client_id: ClientId` but no `org_id: OrgId`. Cases are described as client-scoped in CAP-022 Business Rule: "Cases are scoped by `client_id` — cross-client case access is prevented by the same `OrgSlug` typing (formerly `TenantId`)". This shows the Wave 3 rename is partially reflected in capabilities.md but absent from the story's type definition. The `case:{id}` RocksDB key (line 199) has no OrgId prefix. | Add `org_id: OrgId` to `Case`. Key: `case:{org_id}:{id}`. Reference ADR-006. |
| DRIFT-406-003 | HIGH | I Stale case status enum labels in body | Line 365 reads: `status transitions (\`Open\` → \`InProgress\` → \`Resolved\` → \`Closed\`)`. The canonical 5-state machine is `New → Acknowledged → Investigating → Resolved → Closed` per BC-2.14.002 and Task 1 line 113 in the same file. `Open` and `InProgress` are not valid variant names in the Wave 4 spec. This is an internal body inconsistency that would confuse the implementing agent. | Replace line 365 with the correct 5-state sequence: `New → Acknowledged → Investigating → Resolved → Closed`. |
| DRIFT-406-004 | MEDIUM | D Path inconsistency: case/dedup.rs vs cases/dedup.rs | Token budget line 88, Task 9 line 216, Architecture Mapping lines 359-360, Purity Classification lines 378-379 all reference `prism-operations/src/case/dedup.rs` (no trailing `s`). File Structure Requirements line 440 references `crates/prism-operations/src/cases/dedup.rs` (with `s`). Contradiction within the same story. The rest of the story uses `cases/` as the module name (Task 1 line 104, create_case.rs, update_case.rs, etc.). | Normalise to `cases/dedup.rs` throughout (aligns with cases module). Update all `case/dedup.rs` occurrences to `cases/dedup.rs`. |
| DRIFT-406-005 | MEDIUM | D Architecture path refs | Lines 368, 377 use unqualified `architecture/module-decomposition.md`, `architecture/purity-boundary-map.md`. | Qualify paths. |
| DRIFT-406-006 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"`; `tdd_mode` absent. | Update cycle; add tdd_mode. |

### S-4.07 Case Metrics and Acknowledge Alert

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-407-001 | HIGH | K prism-operations crate | Same as DRIFT-401-001. | Same remediation. |
| DRIFT-407-002 | MEDIUM | D Architecture path refs | Lines 189, 198 use unqualified `architecture/module-decomposition.md`, `architecture/purity-boundary-map.md`. | Qualify paths. |
| DRIFT-407-003 | MEDIUM | B Missing ADR refs | BC-2.14.012 (acknowledge_alert) requires feature flag `CAPABILITY_ACKNOWLEDGE_ALERT` from S-1.08. Story correctly references S-1.08 in Previous Story Intelligence but does not reference ADR-006 for org-scoping of acknowledgment. `acknowledge_alert` must verify that the alert's `org_id` matches the calling session's org context before setting `acknowledged_at`. | Add org-context check to Task 6 and AC-7. Reference ADR-006. |
| DRIFT-407-004 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"`; `tdd_mode` absent; `verification_properties: []` (no VPs assigned). The empty VP list is by design per Dev Notes, but no explicit justification is written as a Gap Register entry. If downstream gate requires VP coverage, this will be flagged. | Update cycle; add tdd_mode. Consider a Gap Register entry explaining zero VPs for this story. |

### S-4.08 Action Delivery Framework

| ID | Severity | Category | Finding | Remediation |
|----|----------|----------|---------|-------------|
| DRIFT-408-001 | HIGH | K prism-operations crate | Same as DRIFT-401-001. | Same remediation. |
| DRIFT-408-002 | HIGH | I OrgId/org-scoping | `ActionSpec` (Task 1 line 115) includes `clients: Vec<ClientId>` but no `org_id`. Rate limit keys (Task 6 line 183: `"{action_id}:fire_count:{hour_bucket}"`) have no OrgId prefix. In multi-tenant, action specs and their state must be org-scoped per ADR-006. The Dev Notes acknowledge `clients = []` means all clients but do not address the org boundary. The architect's ADR identification (DRIFT-arch-001) explicitly flags this as an open architecture question. | Add OrgId to `ActionSpec`. Prefix all `action_state` CF keys with `{org_id}:`. Reference ADR-016 once authored. |
| DRIFT-408-003 | HIGH | J DTU integration surface mismatch | `depends_on: [S-1.15, S-6.11, S-6.12, S-6.13]` (frontmatter line 8). S-6.11 (prism-dtu-slack), S-6.12 (prism-dtu-pagerduty), S-6.13 (prism-dtu-jira) are all `status: merged` with `behavioral_contracts: []` — they expose no formal API surface. The story body claims these DTUs are test fixtures for integration tests (changelog v1.3 line 543: "DTU-first"). The PluginRuntime API (`fire_alert`, `fire_case`, `fire_report`) is correctly anchored to S-1.15 (merged). However, the dependency on the action DTUs as test fixture providers is an untested assumption: the DTU stories do not specify the mock endpoint contract the action engine tests will call. If the DTUs' test harness interface changed post-merge (stub-as-impl pattern disclosed in STORY-INDEX changelog for S-6.12/S-6.13), the action tests could fail. | Review merged S-6.11/12/13 `lib.rs` and `tests/` to confirm the action test fixtures can be satisfied. Add a "Test Fixture Surface" subsection to Previous Story Intelligence documenting which endpoints the tests will use. |
| DRIFT-408-004 | MEDIUM | B ADR-016 not yet authored | Story extensively references `AD-021` (actions.md architecture document) which exists at `.factory/specs/architecture/actions.md`. However, ADR-016 (Action Delivery Framework ADR) does not yet exist — it is on the D-204 authoring list. The story should not be dispatched until ADR-016 is authored, reviewed, and accepted, as it will govern the credential reference model, delivery semantics, and OrgId scoping that the story must implement. | Gate S-4.08 dispatch behind ADR-016 acceptance. Add `anchor_adrs: [ADR-016]` to frontmatter once authored. |
| DRIFT-408-005 | MEDIUM | D Architecture path refs | Lines 365, 374 use unqualified `architecture/module-decomposition.md`, `architecture/purity-boundary-map.md`. | Qualify paths. |
| DRIFT-408-006 | LOW | M Frontmatter schema | `cycle: "v1.0.0-greenfield"`; `tdd_mode` absent. | Update cycle; add tdd_mode. |

---

## Cross-Cutting Drift Classes

| Class | Affected Stories | Recommended Sweep |
|-------|------------------|-------------------|
| K: prism-operations crate does not exist in workspace | All 8 | Story-writer adds workspace registration task to S-4.01 (the crate-creating story); document creation in the other 7 as a prerequisite of S-4.01. |
| I: OrgId/OrgSlug org-scoping absent | All 8 | Add `org_id: OrgId` to all domain types (ScheduleEntry, DiffResult, DetectionRule, Alert, Case, ActionSpec). Prefix all new RocksDB CF keys per ADR-008 universal re-keying rule. Add org-isolation ACs to each story. Reference ADR-006. |
| D: Unqualified `architecture/` path refs in story bodies | All 8 | Replace `architecture/module-decomposition.md`, `architecture/purity-boundary-map.md`, etc. with `.factory/specs/architecture/...` throughout all 8 bodies. Pattern: `Per \`architecture/` → `Per \`.factory/specs/architecture/`. |
| M: Stale `cycle` field and missing `tdd_mode` | All 8 | Set `cycle: "wave-4-operations"` and `tdd_mode: strict` in all 8 story frontmatter files. |
| F: Story-point discrepancies (story vs cycle manifest) | S-4.02 (3 vs 5), S-4.05 (2 vs 1) | Reconcile frontmatter with cycle manifest. The authoritative count should be the story frontmatter; manifest needs correction. |

---

## Implementation Already Done (would fail RED gate trivially)

No Wave 4 behavioral surface has been implemented in the current codebase. The `prism-operations` crate does not exist. All Wave 4 ACs will require fresh implementation. No RED gate trivial-pass risk detected for Wave 4 stories specifically.

However, some Wave 1/2/3 infrastructure that Wave 4 stories depend on is already implemented and could affect story scoping:

| Story / AC | What already exists | Recommendation |
|------------|--------------------|-----------------------|
| S-4.03 BC-2.13.010 (Security UDF Registration) | `prism-query` and `prism-spec-engine` both have architecture comments explicitly prohibiting DataFusion dependency. DataFusion is referenced in S-3.02 (draft) but not yet in any merged crate. The UDF registration story (S-4.03) correctly plans to add DataFusion to `prism-operations` rather than reusing anything that exists. No overlap. | No action needed. |
| S-4.05 / S-4.07 BC-2.14.012 (acknowledge_alert) | S-1.08 (feature flags, merged) provides `CAPABILITY_*` pattern. No alert types exist yet. No overlap. | No action needed. |
| S-4.06 CaseStatus in prism-core | S-1.02 is listed in STORY-INDEX as merged but its status was not directly confirmed in this audit. S-4.06 Previous Story Intelligence says "if S-1.02 does not yet encode the full 12-transition set, add them as part of this story." | Before dispatch, verify prism-core has `CaseStatus` with all 12 transitions and `CaseStatus::can_transition_to()` implemented. If absent, add to S-4.06 scope explicitly. |

---

## Frontmatter Schema Drift

| Story | Field | Expected | Actual |
|-------|-------|----------|--------|
| All 8 | `cycle` | `wave-4-operations` | `v1.0.0-greenfield` |
| All 8 | `tdd_mode` | `strict` | absent |
| All 8 | `traces_to` | Reference to STORY-INDEX.md per VSDD criterion 22 | `[]` (empty) |
| S-4.02 | `points` | 3 (story frontmatter) or 5 (cycle manifest) | Conflict — requires reconciliation |
| S-4.05 | `points` | 2 (story frontmatter) or 1 (cycle manifest) | Conflict — requires reconciliation |
| All 8 | `input-hash` | Hash of current inputs (post-Wave-3 factory state) | `248b3b0` (pre-Wave-3 factory state, predates Wave 3 converge SHA ba3b10c7) |

The `input-hash: "248b3b0"` on all 8 stories predates the Wave 3 converge commit (ba3b10c7). Since all 8 stories were drafted on 2026-04-16/17 and Wave 3 inputs have changed substantially (22 new BCs, 5 new ADRs, OrgId/OrgRegistry, DTU harness), the input hashes are semantically stale. The story-writer must recompute these hashes after drift remediation completes.

---

## Remediation Difficulty Assessment

| Story | Effort | Notes |
|-------|--------|-------|
| S-4.01 | MEDIUM | Org-scoping of ScheduleEntry is the core work. Requires verifying S-3.02 merge state. CI-005 reference in body (line 130) is valid and correct. |
| S-4.02 | LOW-MEDIUM | Org-scoping of diff CF keys. Pack "global" semantics need clarification. Points reconciliation. |
| S-4.03 | MEDIUM | Org-scoping of RuleScope. Missing library entries (regex, arc-swap) are additive. DataFusion version 53 must be confirmed against S-3.02 merge once available. |
| S-4.04 | MEDIUM | Org-prefix on detection_state CF keys. The key encoding scheme (length-prefix bytes \x00/\x01/\x02) must gain an OrgId segment. |
| S-4.05 | LOW-MEDIUM | Add org_id to Alert struct and RocksDB key. Points reconciliation. |
| S-4.06 | MEDIUM | Three changes: org_id on Case, stale `Open/InProgress` enum labels (line 365), and case/dedup.rs path inconsistency. VP-053 Kani proof is in prism-core, not prism-operations — ensure target crate path is correct. |
| S-4.07 | LOW | Minor ADR reference gap; org-context check on acknowledge_alert. |
| S-4.08 | HIGH | Org-scoping of ActionSpec and action_state keys; ADR-016 authoring dependency; DTU test surface audit. This is the most complex story — recommend dispatching last after ADRs 013–016 are accepted. |

---

## Bridge to Architect ADR Findings

The consistency-validator findings **confirm and extend** the architect's ADR identification pass:

**Agreement:** The architect correctly identified the five needed ADRs (ADR-013 through ADR-017/ADR-018). The consistency audit independently identified the same decision gaps through the lens of missing org-scoping (Category I), missing ADR anchors in frontmatter (Category B), and non-existent CI architecture refs in stories.

**Additional drift surfaced that the architect's pass did not flag:**

1. **prism-operations crate absence (K):** The architect's pass reviewed story content but did not verify the workspace Cargo.toml. The crate does not exist and no story task creates it explicitly.

2. **Stale `Open → InProgress` case state machine in S-4.06 body (line 365):** The architect's pass notes CaseStatus is in prism-core but does not flag the contradictory stale label in S-4.06's Architecture Mapping prose.

3. **case/dedup.rs vs cases/dedup.rs path inconsistency within S-4.06:** Four locations say `case/dedup.rs`, one says `cases/dedup.rs`. Intra-story inconsistency not surfaced in architect pass.

4. **DataFusion version anchoring uncertainty:** Stories S-4.03 and S-4.04 specify `datafusion = 53` but DataFusion is not yet present in any merged crate (S-3.02 is draft). The actual version that will land with S-3.02 needs to be confirmed against what the detection stories claim.

5. **EC-12-NNN vs EC-NNN naming inconsistency:** S-4.01 uses a subsystem-prefixed edge case ID format (`EC-12-001`) while all other 7 stories use `EC-001`. This is a LOW cosmetic inconsistency but creates non-uniform convention in the operations domain.

6. **Stale input-hash across all 8 stories (cross-cutting):** `248b3b0` predates the Wave 3 converge. All 8 stories need hash recomputation as part of remediation.

---

## Appendix: Cross-Document Finding DRIFT-manifest-001

The cycle manifest Story Inventory table shows total story points as 45. The per-story frontmatter `points:` values sum to: S-4.01(5) + S-4.02(3) + S-4.03(5) + S-4.04(5) + S-4.05(2) + S-4.06(5) + S-4.07(3) + S-4.08(5) = **33 points**. The manifest's per-story rows for S-4.02 (5 pts vs story's 3) and S-4.05 (1 pt vs story's 2) contribute to the discrepancy. The total "45" may be based on earlier story sizes before the remediation sweeps in pass-60/59. The cycle manifest must be corrected to match authoritative frontmatter totals after reconciliation.
