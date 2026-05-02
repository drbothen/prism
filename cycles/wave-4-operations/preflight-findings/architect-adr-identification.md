---
document_type: preflight-findings
phase: 4.A
producer: architect
timestamp: 2026-05-02T14:00:00Z
inputs:
  - .factory/stories/S-4.01-schedule-crud.md
  - .factory/stories/S-4.02-diff-results-packs.md
  - .factory/stories/S-4.03-detection-rules.md
  - .factory/stories/S-4.04-detection-evaluation.md
  - .factory/stories/S-4.05-alert-generation.md
  - .factory/stories/S-4.06-case-management.md
  - .factory/stories/S-4.07-case-metrics.md
  - .factory/stories/S-4.08-action-delivery.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
  - .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
  - .factory/specs/architecture/decisions/ADR-010-customer-config-schema.md
  - .factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md
  - .factory/specs/architecture/decisions/ADR-012-src-convention.md
  - .factory/specs/architecture/decisions/ADR-014-local-pre-push-ci-gate-asymmetry.md
  - .factory/cycles/wave-4-operations/cycle-manifest.md
---

# Wave 4 ADR Identification (DISCOVERY ONLY)

## Summary

5 new ADRs needed; 0 existing ADRs to amend; 4 decision areas covered by existing ADRs without modification.

The D-204 candidate list (ADR-013 schedule, ADR-014 detection, ADR-015 action, ADR-016 case) requires renumbering because ADR-014 is taken (Local Pre-Push CI Gate). The renumbered set is: ADR-013 (Schedule), ADR-015 (Detection Rule Language), ADR-016 (Action Delivery Framework), ADR-017 (Case State Machine). An additional ADR-018 for the Differential Result Pack format is recommended because the pack/epoch storage contract introduces a novel RocksDB column family design decision that is not covered by ADR-008's DTU state segregation scope.

---

## New ADRs Proposed

| ADR # | Title | Decision Area | Rationale | Anchor Stories | Aligns With |
|-------|-------|---------------|-----------|----------------|-------------|
| ADR-013 | Schedule Execution Semantics — Tick Interval, Splay, In-Flight Skip, and Missed-Fire Policy | Schedule execution | S-4.01 introduces a 60-second tick loop with deterministic splay, global semaphore (16 permits), and explicit skip-not-queue missed-fire policy. These semantics affect VP-026 (splay determinism), VP-030 (cap enforcement), shared-semaphore coupling to S-4.08, and memory budget allocation. No existing ADR covers the scheduling runtime model. Directly analogous in scope to ADR-006 (multi-tenant topology) for the operations layer. | S-4.01, S-4.08 | ADR-008 (semaphore shared with action delivery state), ADR-010 (config-driven `PRISM_SCHEDULER_TICK_SECS` env convention), ADR-012 (prism-operations crate layout) |
| ADR-015 | Detection Rule Language — DSL Syntax, Three-Scope Resolution, Compilation Strategy, and UDF Registry | Detection rule language | S-4.03/S-4.04 introduce a TOML-based `.detect` file format, a `RuleCondition` enum (Single/Correlation/Sequence), a rule-to-DataFusion-SQL compiler, security UDFs (`subnet_contains`, `ioc_match`, `time_window`), and a three-scope merge (Global/Client/Analyst). This is a novel DSL with formal compilation semantics. VP-018 covers compilation determinism. No existing ADR defines the language or compilation contract. The IOC pattern store (`ArcSwap<Arc<PatternStore>>`) introduces a new shared-state pattern not covered by ADR-008. | S-4.03, S-4.04, S-4.05 | ADR-006 (OrgId-scoped rule resolution — Global/Client maps to OrgId boundaries), ADR-008 (detection_state CF key encoding — extends state segregation pattern), ADR-010 (built-in sensor config-driven philosophy — `.detect` files are the analogous config artifact for detection) |
| ADR-016 | Action Delivery Framework — Trigger Model, Delivery Semantics, Credential Reference Model, and WASM Plugin Delegation | Action delivery | S-4.08 introduces `.action.toml` spec files, four trigger modes (Alert/Case/Schedule/Manual), three built-in destination types (webhook/email/syslog), at-least-once vs best-effort vs fire-and-forget delivery semantics per trigger mode, exponential backoff retry bounded to 5 attempts (VP-044), WASM plugin delegation, injection-scanned template rendering (BC-2.09.004), inline credential rejection (VP-046, E-ACTION-001), and UUID v7 validation for interpolated values (VP-047). The `action_state` CF is new and requires a keying scheme. These decisions collectively define the outbound side of the Prism operations platform. They touch ADR-010's credential reference model (`.action.toml` uses the same opaque reference scheme as customer configs) and extend it to a new context. | S-4.08, S-4.05, S-4.06 | ADR-010 (credential reference schemes — `.action.toml` MUST use same `vault://`, `env://`, `file://`, `keyring://` opaque reference model), ADR-008 (action_state CF key design extends DTU state segregation pattern), ADR-006 (OrgId scoping — action client filters use OrgId routing) |
| ADR-017 | Case State Machine — 5-State Enum, 12 Valid Transitions, Disposition Enforcement, and TTR Timestamp Semantics | Case state machine | S-4.06 introduces a closed 5-state machine (New/Acknowledged/Investigating/Resolved/Closed) with exactly 12 valid transitions (including two reopen paths), mandatory disposition on Resolved, and first-resolution-timestamp semantics for TTR correctness across reopen cycles (VP-054). VP-053 formally verifies that no Case can reach Resolved without a disposition. The state machine is a core domain invariant enforced in `prism-core` (`CaseStatus::can_transition_to()` is a pure function). No existing ADR covers the operations domain state machines. This ADR is the canonical reference for the transition table and disposition enforcement rule, enabling downstream stories (S-4.07 metrics, S-5.x) to anchor against it. | S-4.06, S-4.07 | ADR-006 (OrgId scoping — cases are org-scoped per Wave 3 multi-tenant topology), ADR-008 (cases CF in RocksDB — extends state segregation principles to operations domain) |
| ADR-018 | Differential Result Pack Format — Row Hashing, Epoch/Counter Semantics, Pack TOML Schema, and Capability-Gated Execution | Differential result pack format | S-4.02 introduces a novel data structure: the `DiffResult` (added/removed sets between schedule executions keyed by `blake3(canonical_json(row))`), an epoch-counter with merge-operator semantics for exactly-once advancement, the `diff_results` CF with 200MB cap and zstd compression, and the `.pack.toml` format for grouping schedules with per-capability conditional execution gates. These design decisions are non-obvious: the canonical JSON normalization choice (not raw bytes) directly governs VP-019 correctness; the merge-operator atomicity choice prevents TOCTOU epoch races; the pack-expands-to-ScheduleEntry-at-load-time decision (not runtime) is a deliberate architecture choice. None of these are covered by ADR-008 or ADR-010. This ADR is a borderline case — it could be merged into ADR-013 as a section on persistence semantics — but the pack format and epoch/counter design are sufficiently independent of the tick loop to warrant a separate document. The spec-first convergence process should weigh whether to combine with ADR-013. | S-4.02, S-4.01 | ADR-013 (diff results are produced by the schedule execution loop), ADR-008 (diff_results CF follows DTU state segregation principles), ADR-010 (`.pack.toml` files are config-driven artifacts analogous to customer config TOML files) |

---

## Amendments to Existing ADRs (if any)

None. Wave 4 decisions are sufficiently novel that new ADRs are preferred over amendments. Rationale: all accepted Wave 3 ADRs (ADR-006 through ADR-014) are ACCEPTED with IMPLEMENTED status post-Wave-3 closure. Amending an IMPLEMENTED ADR to cover a new wave's decisions would conflate two wave-cycles in one document, making traceability and blame harder. New ADRs with explicit `aligns_with` references to Wave 3 ADRs maintain clean wave-boundary traceability.

One exception to flag: **ADR-010 credential reference model** is extended by ADR-016. ADR-016 should explicitly reference ADR-010 §2.3.1 (allowed opaque reference schemes) and state that `.action.toml` credential references follow the same four-scheme model. The spec in ADR-010 is unchanged; ADR-016 cross-references it rather than amending it.

---

## Decisions Covered Without New ADR

| Decision | Existing ADR | Why sufficient |
|----------|--------------|----------------|
| OrgId scoping of operations objects (schedules, rules, cases, actions are all org-scoped) | ADR-006 §2.1 (OrgId is the canonical internal routing key for all org-scoped objects) | Wave 4 objects (ScheduleEntry, DetectionRule, Case, Alert) use `OrgId` as their scope key by inheritance from the Wave 3 identity model. No new decision is needed — the pattern is already established and Wave 4 stories already reference `ClientId` (which maps to `OrgId` after the TenantId→OrgSlug alias removal). |
| RocksDB column family introduction for new Wave 4 CFs (`schedules`, `diff_results`, `detection_rules`, `detection_state`, `alerts`, `cases`, `action_state`) | ADR-008 §2.1 (universal re-keying rule and CF access patterns) + S-2.01 CF registration | ADR-008 establishes the CF design principles; individual CF introductions are story-level decisions already documented in BC-2.12.010, BC-2.13.012, BC-2.14.009. No architectural re-decision required — the pattern is established. |
| Feature flag gating of detection and action delivery (`FEATURE_DETECTION_ENGINE`, `FEATURE_AUTO_CASE_CREATION`, `CAPABILITY_ACKNOWLEDGE_ALERT`) | S-1.08 feature flag subsystem (Wave 1) | Feature flag gating is an established pattern from Wave 1. No new ADR needed for individual flag definitions. |
| `prism-operations` crate layout and src/ conventions | ADR-012 (workspace src/ convention normalization) | ADR-012 establishes the canonical crate layout for all workspace members. The `prism-operations` crate follows the same conventions without any decision variance. |

---

## Open Architecture Questions Surfaced

- **ADR-013 or ADR-018: combine vs split.** The differential results / epoch semantics (S-4.02) are tightly coupled to the schedule execution loop (S-4.01). A single ADR-013 with two sections (execution semantics + persistence semantics) may be more cohesive than two separate ADRs. The spec-first phasing convergence should make this call before ADR authoring begins.

- **TenantId alias removal timing.** Cycle manifest §Deprecations notes that the `TenantId` alias (Wave 3 D-157 deferral, one-wave alias) is to be removed in Wave 4. All 8 Wave 4 stories use `ClientId` in their narratives (a mix of `OrgId` and the old `TenantId` alias). The spec-drift audit (separate pre-flight item) must determine whether story bodies need `ClientId` → `OrgId` substitution before dispatch.

- **Shared semaphore scope (S-4.01 + S-4.08).** The 16-permit schedule semaphore is explicitly shared between schedule execution (S-4.01) and action report queries (S-4.08). ADR-013 must document the semaphore ownership model: who constructs it, who holds the `Arc<Semaphore>`, and what happens if the semaphore is exhausted by one subsystem while the other is starved. This is a liveness property that warrants a formal VP.

- **`CaseStatus::can_transition_to()` in `prism-core` vs `prism-operations`.** S-4.06 states the 12-transition function is in `prism-core` (S-1.02). ADR-017 must verify this is actually the case in the Wave 3-converged codebase before authoring the transition table. If `CaseStatus` was not fully specified in Wave 1 (S-1.02 may have been a stub), the ADR author needs to extend `prism-core` as part of Wave 4 spec work.

- **Action delivery OrgId scoping.** S-4.08's `ActionSpec` has a `clients: Vec<ClientId>` filter list. The ADR-016 threat model should address whether an empty `clients = []` list is "all clients" or "no clients" (the story Dev Notes say "all clients" but this must be formalized as an explicit invariant, not just a dev note).

- **Alert deduplication window default linkage to schedule interval.** S-4.04 states the dedup window defaults to "the interval of the parent schedule, or 1 hour if no parent schedule." This creates a runtime dependency between the detection state and the schedule state. ADR-015 should formalize whether this linkage is a scheduling-time resolution or a runtime lookup.

- **ADR-018 borderline: merge with ADR-013 or keep separate.** If combined, ADR-013 becomes the primary Schedule + Differential Operations ADR (~2,500 tokens). If split, ADR-018 is modest in scope (~1,000 tokens for the pack/epoch design decisions). The prefer-amendment heuristic slightly favors combining since the diff engine is a direct output of the schedule executor. This is flagged for the spec-first convergence decision.

---

## Recommended Authoring Order

The dependency graph requires ADR-013 before ADR-015/ADR-018, because detection evaluation depends on schedule execution producing differential results. ADR-017 is independent of ADR-015 (case management does not depend on the detection rule language). ADR-016 depends on ADR-013 (shared semaphore) and ADR-015 (alert broadcast trigger source).

```
PHASE 1 (no intra-Wave-4 dependencies):
  ADR-013 — Schedule Execution Semantics
  ADR-017 — Case State Machine          (purely domain-model; independent of scheduler)

PHASE 2 (depends on ADR-013):
  ADR-018 — Differential Result Pack Format   (produced by schedule executor)
  ADR-015 — Detection Rule Language           (evaluation triggered by schedule executor)

PHASE 3 (depends on ADR-013 + ADR-015):
  ADR-016 — Action Delivery Framework         (subscribes to alert broadcast from detection)
```

Note: if ADR-018 is merged into ADR-013 (per the "combine" option in Open Questions), PHASE 2 collapses to a single ADR-015 dispatch. The authoring order is otherwise unchanged.

All five ADRs (or four if combined) follow the full VSDD process per D-202: 3-clean adversarial convergence, consistency-validator, spec-reviewer, input-hash, human approval gate before any story dispatch.
