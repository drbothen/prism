---
document_type: vsdd-plugin-tech-debt-register
level: ops
version: "2026-05-05T00:00:00Z"
status: current
producer: state-manager
timestamp: 2026-05-05T00:00:00Z
project: prism (carved out from main tech-debt-register)
scope: vsdd-factory plugin / methodology / pipeline mechanics
deferred_to: vsdd-factory plugin maintenance cycle (separate-repo / cross-session)
---

# VSDD Plugin Tech Debt Register

## Purpose

Carved out from `.factory/tech-debt-register.md` per user directive 2026-05-02. VSDD-related items are pipeline/methodology debt and will NOT be addressed during Wave 4 execution. This file is a holding queue for the next vsdd-factory plugin maintenance session.

## Scope Criteria

Items included here meet one or more of the following criteria:
- VSDD plugin defects (agent dispatch, tool binding, skill bugs)
- Pipeline mechanics improvements (state-manager guardrails, two-commit protocol enhancements, gate-step pass-N policy)
- Process-gap codifications (linter additions, skill checklist additions)
- VSDD-factory skill / template / agent definition changes

## Out of Scope (stays in main `tech-debt-register.md`)

- Product code findings
- Infrastructure (CI/clippy)
- Test coverage gaps in the product
- Holdout scenario refreshes
- Security findings
- Capability TDs (TD-W4-AUDIT-QUERY-REPLAY-001, TD-W4-LOG-FORWARDING-001, TD-W4-ALERTING-WORKFLOWS-001, etc.)

---

## Debt Items

| ID | Source | Description | Priority | Introduced | Target |
|----|--------|-------------|----------|-----------|--------|
| TD-VSDD-001 | Wave 2 stub-as-impl anti-pattern (Layer 1) | Add anti-precedent guard text to vsdd-factory deliver-story SKILL.md and per-story-delivery.md so stub-architect agents don't copy stub-as-impl patterns from sibling crates. Identified in Wave 2 parallel batch: S-2.04/S-6.12/S-6.13 copied prism-dtu-armis and other Wave 1 DTU crate precedents. 3 of 5 stories affected. | P2 | wave-2-parallel-batch | vsdd-factory plugin maintenance |
| TD-VSDD-002 | Wave 2 stub-as-impl anti-pattern (Layer 2) | Add Red Gate density check to vsdd-factory per-story-delivery.md as a mandatory orchestrator gate between Step 3 (Red Gate) and Step 4 (Implementer). Threshold: RED_TESTS / TOTAL_NEW_TESTS >= 0.5 unless documented. Catches stub-as-impl before implementer dispatch becomes no-op. | P2 | wave-2-parallel-batch | vsdd-factory plugin maintenance |
| TD-VSDD-003 | Wave 2 stub-as-impl anti-pattern (Layer 3) | Add `tdd_mode: strict \| facade` frontmatter field to vsdd-factory story template; route facade-mode stories through different per-story-delivery flow with explicit acknowledgment and mutation testing gate. | P2 | wave-2-parallel-batch | vsdd-factory plugin maintenance |
| TD-VSDD-004 | Wave 2 stub-as-impl anti-pattern (Layer 4) | Wire mutation testing gate (cargo mutants >= 80% kill rate) into vsdd-factory wave-gate skill for facade-mode stories (tdd_mode: facade). Validates that stub-as-impl tests catch real regressions before wave gate closes. | P2 | wave-2-parallel-batch | vsdd-factory plugin maintenance |
| TD-VSDD-005 | Wave 2 gate Pass 2 adversary dispatch failure | vsdd-factory:adversary subagent has a runtime tool-binding defect. Agent definition declares `Tools: Read, Grep, Glob` but at runtime only `Read` is bound. Pass 2 had to fall back to general-purpose-as-adversary workaround. This blocks the canonical vsdd-factory adversarial discipline. Accumulating alongside earlier session Skill-tool-empty-body bug (fix-prompt deleted per user request). These are vsdd-factory plugin-level defects to address during the housekeeping pause before Wave 3. | P2 | wave-2-gate-pass-2 | Before next adversarial review (Wave 4 gate at latest) |
| TD-W2-PASS1-TOOLING-001 | Wave 2 gate Pass 1 process-gap disclosure | The Pass 1 adversary ran with Read-only tool access (no Glob/Grep/Bash), which prevented full verification of policies POL-1, POL-2, POL-5, POL-6, POL-7, POL-8, POL-9. Pass 2+ must dispatch with full tool access. Investigate root cause: agent definition declares `Tools: Read, Grep, Glob, Bash` but only Read was operative in this session. May be a session-specific harness issue or a bug in the orchestrator's adversary dispatch path. Root cause is VSDD plugin dispatch — orchestrator/adversary dispatch path. | P2 | wave-2-gate-pass-1 | Before next adversarial review (Wave 4 gate at latest) |
| TD-VSDD-029 | Pass 35 adversary review (M-35-001) | Add a guardrail clause to state-manager.md (vsdd-factory plugin) requiring that STORY-INDEX version bumps update BOTH the prose-bullet changelog AND the tabular changelog. Optionally extend Defensive Sweep Discipline section with parallel-form symmetry sweep, or add as 6th anti-pattern in Wave-gate remediation bursts. Detected via Pass 33→34→35 sequence: M-33-001 added v1.64 to tabular only; M-34-001 backfilled prose; M-35-001 surfaced absence of guardrail. Recurrence prevention. | P3 | wave-3-phase-3a-pass-35 | vsdd-factory plugin maintenance cycle |
| TD-VSDD-030 | pass-48 PG-48-001 | ADR §2 Status block ↔ frontmatter status linter — verify §2 body Status text matches frontmatter status field; surfaced by PG-48-001 (7 ADRs had stale PROPOSED body when frontmatter was ACCEPTED). Separate-repo vsdd-factory plugin fix. | P3 | wave-3-integration-gate-pass-48 | vsdd-factory plugin maintenance cycle |
| TD-VSDD-031 | pass-48 PG-48-002 | cycle-manifest epic membership ↔ story epic_id linter — verify each story's epic_id frontmatter matches the epic-view table it appears in; surfaced by PG-48-002 (W3-FIX-WIN-001 had epic_id E-3.3 but appeared in E-3.5 table). Separate-repo vsdd-factory plugin fix. | P3 | wave-3-integration-gate-pass-48 | vsdd-factory plugin maintenance cycle |
| TD-VSDD-032 | pass-50 PG-50-001 | Adversary review file persistence guard required. Reports generated in-chat by adversary agent but not persisted to factory-artifacts when the state-manager burst is not run in the same session. Two-Commit Protocol does not include a checkpoint for adversary file persistence. Add mandatory adversary file persistence step to wave-gate skill checklist. Surfaced by L-50-002 (pass-48/49 reports missing). | P3 | wave-3-integration-gate-pass-50 | vsdd-factory wave-gate skill maintenance |
| TD-VSDD-033 | pass-50 PG-50-003 | AC scope-coverage matrix template requirement — no standard template for verifying all story ACs are covered by tests. The consistency-validator has no check for AC-to-test mapping coverage. Several stories lack explicit AC-to-test mapping. Add AC scope-coverage matrix template to story template and consistency-validator checklist. Surfaced by PG-50-003. | P3 | wave-3-integration-gate-pass-50 | vsdd-factory story template + consistency-validator skill maintenance |
| TD-VSDD-034 | pass-53 PG-53-001 | gate-step pass-N completeness policy for non-impacted steps. When a gate pass advances to pass-N, non-impacted gate steps that have sustained verdicts from pass-N-1 should either (a) produce a brief pass-N confirmation report or (b) have a policy decision recorded that prior pass's verdict carries forward without re-evaluation. Surfaced by PG-53-001 in pass-53. Separate-repo vsdd-factory plugin fix. | P3 | wave-3-integration-gate-pass-53 | vsdd-factory wave-gate skill + policy registry maintenance |
| TD-W2-FIXK-001 | W2-FIX-K P7 process-gap (HIGH-001/003) | Pass 7 HIGH-001 (token_id in persisted audit entry violating BC-2.05.010 TV) and HIGH-003 (tautology test) revealed two gaps in the `validate-consistency` skill: (a) no tautology-detector to flag `test_BC_*` functions that don't call the corresponding `emit_*` function; (b) no BC-TV field-exclusion checker to parse canonical TV tables for field-level exclusion markers and cross-reference with struct definitions and test coverage. Root cause is the vsdd-factory validate-consistency skill. Recommend extending `validate-consistency` with both checks to prevent recurrence. Estimated effort: 1 day. | P3 | W2-FIX-K | vsdd-factory validate-consistency skill maintenance |
| TD-VSDD-035 | wave-4-operations pre-flight (2026-05-02); user-flagged methodology innovation | Pre-flight cycle-manifest pattern — formalize wave-kickoff artifact as a vsdd-factory skill. Currently authored ad-hoc by orchestrator. Pattern includes: charter section, story inventory table, topology/dispatch order diagram, pre-flight blocking checklist, spec-first decision section, architecture gates, convergence targets, open questions for human approval, resume steps. Wave 4 was the first wave to receive a pre-flight cycle-manifest before story dispatch (.factory/cycles/wave-4-operations/cycle-manifest.md, 0cd3565d). Wave 3 kicked off via D-040..D-046 decisions only — no pre-flight artifact existed. Codify as `/vsdd-factory:author-wave-preflight` skill with template at `.factory/templates/wave-preflight-template.md`. | P3 | wave-4-operations pre-flight (2026-05-02) | vsdd-factory plugin maintenance cycle |
| TD-VSDD-036 | wave-4-operations pre-flight (2026-05-02); first surfaced as D-045 in Wave 3 (2026-04-27); user-flagged methodology innovation | Spec-first phasing (Phase N.A BLOCKING per D-045) is currently a per-wave decision made ad-hoc. Wave 3 was BLOCKING; Wave 4 question is open. Formalize as a wave-kickoff policy with explicit human approval gate and templated decision rationale. Should be a question in the wave-preflight cycle-manifest (Open Questions section) with explicit options: (a) BLOCKING — full Phase N.A spec convergence required, (b) DRIFT-AUDIT-ONLY — verify existing drafts align with current architecture, (c) NON-BLOCKING — proceed directly to story implementation. Add to `/vsdd-factory:author-wave-preflight` skill template + policy registry. | P3 | wave-4-operations pre-flight (2026-05-02) | vsdd-factory wave-gate skill + policy registry maintenance cycle |
| TD-VSDD-037 | wave-3-multi-tenant closure + Wave 4 pre-flight (2026-05-02); user-flagged methodology innovation | Cross-wave carry-forward debt bucketing at gate close is currently ad-hoc. Wave 3 closure required manual categorization of TD items into: (a) fix-wave-N+1 candidates, (b) deferred to later wave, (c) stay-in-product-register, (d) extract-to-vsdd-plugin-tech-debt. Codify as a state-manager step at gate-close: each open TD must be tagged with one of the four buckets. Pattern emerged organically in Wave 3 closure (W3.4 fix wave, vsdd-plugin-tech-debt.md extraction). Add as a mandatory section in cycle-manifest closure blocks: 'Carry-Forward Debt Bucketing' with table mapping TD IDs to buckets. | P3 | wave-3-multi-tenant closure + Wave 4 pre-flight (2026-05-02) | vsdd-factory state-manager skill maintenance cycle |
| TD-VSDD-040 | Wave 4 Phase 4.A Pass 3 + Pass 5 + pre-Pass-14 — 3rd chain-corruption occurrence | state-manager two-commit-protocol chain-corruption recurring pattern. Three prior occurrences (Pass 3, Pass 5, pre-Pass-14): root cause is hook re-detecting stale citation after each SHA-fix commit, creating an infinite-fixup chain. Symptom: `verify-sha-currency.sh` reports stale SHA in STATE.md/SESSION-HANDOFF.md after Stage 2 push, tempting a 3rd commit. Suggested fix: (a) atomic Stage 2 approach — write placeholder SHA `15fa97e6` in Stage 1, then amend Stage 2 in-place with real SHA (no 3rd commit needed); or (b) hook suppression with `SKIP_SHA_CHECK=1` mid-burst exclusively for the Stage 2 amend operation. Either eliminates the multi-commit fixup chain. | P2 | wave-4-phase-4a-pass14 (2026-05-03) | vsdd-factory state-manager skill + verify-sha-currency.sh hook maintenance |
| TD-VSDD-041 | Wave 4 Phase 4.A Pass 14 F-P14-H-001 — pre-pass sweep missed audit-event terminology class | Pre-pass sweep methodology (TD-VSDD-039) currently checks CF-key prefix order and VP module-column cross-check but does NOT check audit-event-terminology consistency: ADR §X.Y declared event names vs story Task body emit-call names. F-P14-H-001 (ScheduleFireSkipped vs ScheduleFireMissed{miss_reason:SemaphoreExhausted} in S-4.01) would have been caught by this check. Recommend extending standard pre-pass sweep checklist: (1) for each ADR in scope, grep §X.Y Event Taxonomy / audit event sections for declared event token names; (2) for each story in scope, grep Task body + EC emit-call literals; (3) flag any mismatch as HIGH candidate before adversary dispatch. | P2 | wave-4-phase-4a-pass14 (2026-05-03) | vsdd-factory sweep skill checklist maintenance |
| TD-VSDD-042 | Wave 4 Phase 4.A Pass 15 F-P15-H-002 — STORY-INDEX top-level aggregates not in standard POLICY 9 cascade checklist | STORY-INDEX.md `total_vps_assigned` frontmatter field and the matching prose overview VP breakdown bullet are not in the standard POLICY 9 cascade checklist for VP-addition bursts. Wave 4 ADR-burst (2026-05-02) added VP-137..VP-145 (9 VPs); VP-INDEX, verification-architecture, and verification-coverage-matrix were all updated correctly, but STORY-INDEX aggregates were missed and drifted for 14 passes before detection. Extend POLICY 9 propagation cascade to include: (4) STORY-INDEX.md `total_vps_assigned:` frontmatter field and (5) STORY-INDEX.md prose overview VP breakdown (count + per-type tallies) on every burst that adds or removes VPs. Hook recommendation: extend `validate-vp-consistency.sh` (POLICY 9 lint hook) to also verify STORY-INDEX aggregates against VP-INDEX totals. Discovered: Pass 15 (Wave 4 Phase 4.A). | P2 | wave-4-phase-4a-pass15 (2026-05-03) | vsdd-factory validate-vp-consistency.sh hook + POLICY 9 checklist maintenance |
| TD-VSDD-043 | Wave 4 Phase 4.A Pass 16 F-P16-M-002 | ADR Status H2 sync requires structural lint-hook enforcement, not textual checklist. TD-VSDD-039 codification proved insufficient for cascade bursts (ADR-015+018 drifted despite checklist existing). Recommendation: `validate-adr-status-sync.sh` pre-commit hook for decisions/ directory. | P2 | wave-4-phase-4a-pass16 (2026-05-03) | vsdd-factory validate-adr-status-sync.sh hook + pre-commit pipeline maintenance |
| TD-VSDD-044 | Wave 4 Phase 4.A pre-Pass-17 cite-repair burst (2026-05-03) | state-manager commit-burst Stage 2 code path must update BOTH STATE.md and HANDOFF.md `factory-artifacts canonical SHA:` fields uniformly. Discovered: pre-Pass-17 burst (Wave 4 Phase 4.A) updated HANDOFF only; STATE was missed; hook validate-wave-gate-prerequisite.sh blocked subsequent operations. Severity: MEDIUM. Hook recommendation: state-burst skill should grep both STATE.md and HANDOFF.md for SHA-cite fields and update all matches in lockstep. | P2 | wave-4-phase-4a-pre-pass17-cite-repair (2026-05-03) | vsdd-factory state-burst skill + Stage 2 backfill code path maintenance |
| TD-VSDD-045 | Wave 4 Phase 4.A Pass 17 F-P17-M-002 — STORY-INDEX VP Assignment Matrix missing W3/W4 VPs | STORY-INDEX VP Assignment Matrix stops at VP-062. Wave 3 added VP-063..VP-136 and Wave 4 added VP-137..VP-145; these 83 VPs have no rows in the matrix. The gap is structural — requires a major rebuild of the matrix section, not a targeted fix. Severity: MEDIUM (cosmetic; does not affect implementation correctness; VP-INDEX and verification-architecture are the authoritative coverage sources). Discovered: Pass 17 (Wave 4 Phase 4.A). Remediation: deferred to post-Phase-4.A convergence cleanup or Wave 5 baseline. | P3 | wave-4-phase-4a-pass17 (2026-05-03) | state-manager: STORY-INDEX VP Assignment Matrix rebuild at next major index maintenance window |
| TD-VSDD-047 | Wave 4 Phase 4.A Pass 22 F-P22-H-001 — CF key format fixes must grep all architecture docs for the same CF name in lockstep | When fixing a CF key format, grep all architecture docs for that CF name and audit all key-format tables in lockstep. Discovered: Pass 22 (actions.md §"Delivery state" had 4 stale rows even after Pre-Pass-21 broad sweep + Pass 21 fix to data-layer.md). The Pre-Pass-21 broad-sweep fixed actions.md surface-level claims (names, constants) but did not audit the action_state CF key table against ADR-016 §2.5 canonical form. Severity: MEDIUM. Hook recommendation: when state-manager bumps a CF-related doc, grep for CF name + key-format table patterns across all architecture/*.md and BC files; flag any non-canonical format. Pattern: `:{action_id}:\|:{schedule_id}:\|:diff:\|:case:\|:alert:\|:retry:\|:dedup:` | P2 | wave-4-phase-4a-pass22 (2026-05-03) | vsdd-factory pre-pass sweep checklist + CF-key-format audit class extension |
| TD-VSDD-048 | Wave 4 Phase 4.A Pass 23 F-P23-L-001 — Broad-sweep methodology must include exhaustive grep-completeness check | Broad-sweep target lists are hand-curated with no mechanical enforcement of exhaustiveness. Discovered: Pass 23 — Pre-Pass-21 sweep target list missed operational-pipeline.md, allowing 3 stale references (16-permit, Action Engine, 1-second tick) to survive two pre-pass sweeps and be caught only in Pass 23. Severity: MEDIUM. Hook recommendation: at end of every broad-sweep burst, run grep for canonical stale tokens (16-permit, 16 max, 16 concurrent, Action Engine, 1-second tick, ActionEngine[^a-zA-Z]) across ALL specs/architecture/*.md and abort if hits found. Replaces hand-curated target lists with mechanical completeness verification. | P2 | wave-4-phase-4a-pass23 (2026-05-04) | vsdd-factory pre-pass sweep skill + grep-completeness enforcement hook |
| TD-VSDD-049 | Wave 4 Phase 4.A Pass 24 — Comprehensive PRD §2 BC-table↔BC H1 byte-equal sync check | Comprehensive PRD §2 BC-table↔BC H1 byte-equal sync check. Discovered: Pass 24 — pre-Pass-24 sweep (TD-VSDD-048) fixed prd.md §X INV-ACTION-004 prose but a separate PRD §2 BC table cell (BC-2.18.004 title column) still contained the superseded title "Scheduled Report Queries — try_acquire() on 16-Permit Semaphore" while the canonical BC H1 is "Action Delivery Semaphore — 8-Permit Independent Pool". These are two different drift classes: (1) root-contract prose drift (caught by TD-VSDD-048 grep) and (2) BC table cell title drift (not caught by grep on architecture tokens). Severity: HIGH. Hook recommendation: at end of EVERY PRD-affecting burst AND every BC-H1-renaming burst, mechanically extract all `| [BC-X.YY.ZZZ](...) | <title> |` rows from PRD §2 BC table and compare each title to the corresponding BC file H1; abort if any mismatch. Result of first run (Pass 24): 1/200 drift found and fixed (prd.md v1.9). | P2 | wave-4-phase-4a-pass24 (2026-05-04) | vsdd-factory PRD §2 BC-table title sync hook + pre-pass sweep checklist extension |
| TD-VSDD-050 | Wave 4 Phase 4.A Pass 25 — PRD §2 SUBSYSTEM PROSE sync check (sibling to TD-VSDD-049 BC-table sync) | PRD §2 SUBSYSTEM PROSE sync check. Discovered: Pass 25 — stale `action_dispatcher` token in PRD §2 line 382 subsystem-introduction prose paragraph escaped TD-VSDD-049 (which only checked BC table CELL title columns, not surrounding prose paragraphs). Two distinct content regions in PRD §2: (1) BC table rows (covered by TD-VSDD-049) and (2) subsystem-introduction prose paragraphs (this item). Severity: MEDIUM. Hook recommendation: scan PRD §2 subsystem-introduction prose paragraphs for module-name + architectural-name tokens against concurrency-architecture.md and module-decomposition.md canonicals; flag unknown module-name tokens as HIGH candidates before adversary dispatch. Bonus codification: orchestrator-authored fix-burst prompts that introduce factual claims (module names, type names, paths) MUST be verified against architecture canonicals before dispatch — fresh-context Pass 25 caught an orphan introduced by the orchestrator's own pre-Pass-24 fix-burst prompt. | P2 | wave-4-phase-4a-pass25 (2026-05-04) | vsdd-factory PRD §2 subsystem-prose sync hook + orchestrator-prompt verification discipline |
| TD-VSDD-051 | Wave 4 Phase 4.A Pass 26 — Orchestrator-prompt-introduced factual claim verification + sibling-ADR prose sweep. Discovered: Pass 26 found `action_dispatcher` orphan in ADR-016 (sibling regression of Pass 25 PRD fix); Pre-Pass-27 sweep then found same orphan in vp-045 spec (3 more sites). Total 5 orphan sites across 3 docs (PRD, ADR-016, vp-045 spec) all introduced by orchestrator-authored fix-burst prompt text. Severity: HIGH. Hook recommendations: (a) pre-dispatch verification: grep orchestrator's fix-burst prompt text for module names + type names + paths; cross-check against canonical glossary (concurrency-architecture + module-decomposition). (b) sibling-ADR prose sweep: when a drift class is closed in PRD or BC, automatically scan all sibling ADRs (§5 Verification Plan, §-Remediation-Notes) for same orphan token. Sister of TD-VSDD-049/050. | P2 | wave-4-phase-4a-pass26 (2026-05-04) | vsdd-factory orchestrator-prompt verification hook + sibling-ADR prose sweep class maintenance |
| TD-VSDD-052 | Wave 4 Phase 4.A Pass 27 — Pre-dispatch VP scope verification. Sister to TD-VSDD-051 (orchestrator-prompt module-name verification). Discovered: Pass 27 — Pass 20 F-P20-H-002 fix-burst prompt authored "VP-047 verifies action delivery dedup correctness" but VP-047 actually anchors UUID v7 validation per VP-INDEX line 68. The erroneous rationale was transcribed verbatim into ADR-016 §5.4 footer and v0.12 changelog entry without any cross-check against VP-INDEX's authoritative `Property` field. NEW class beyond stale module names: semantic mis-anchor in VP rationale text. Severity: HIGH. Hook recommendation: when orchestrator's fix-burst prompt mentions a VP-NNN with explanatory rationale, automatically grep VP-INDEX for VP-NNN's authoritative `Property` field and require the rationale wording to mention canonical Property terms (e.g., for VP-047: 'UUID v7 validation' must appear); abort dispatch otherwise. | P2 | wave-4-phase-4a-pass27 (2026-05-04) | vsdd-factory orchestrator-prompt VP-scope verification hook maintenance |
| TD-VSDD-053 | Wave 4 Phase 4.A R9 gate (2026-05-04) — Structural fix for TD-VSDD-044 (manifested 6x in single session). Self-referential factory-artifacts HEAD SHA cites in STATE.md (3 sites) + SESSION-HANDOFF.md (5+ sites) create infinite "fix-the-fix" chains in two-commit protocol Stage 2 backfill. Defect class is NARROW: only the *current-burst* HEAD SHA cite loops. Other SHA usage (input-hash drift detection, historical changelog SHA references, decisions-log audit trail, cycle-manifest history) is stable and must be preserved. Severity: HIGH. Fix: convert state-burst to single-commit protocol; remove the 8 specific self-referential cite sites from STATE.md/HANDOFF.md templates; verify-sha-currency.sh hook stops checking STATE/HANDOFF "current" cite (keeps actual git HEAD validation). Estimated savings: ~5 force-pushes + 30+ minutes per session of similar scope. RESOLVES-BY: TD-VSDD-044. | P0 | wave-4-phase-4a-r9-gate (2026-05-04) | vsdd-factory plugin: hooks/verify-sha-currency.sh + skills/state-burst/SKILL.md + STATE.md/SESSION-HANDOFF.md templates |
| TD-W4-RETRY-OBS-001 | Wave 4 Phase 4.A R8 spec-reviewer iter-3 (2026-05-04) — SR-LOW-001: ActionDeliveryEngine `RetryState` struct (ADR-016 §2.5) is missing `first_attempted_at: Timestamp` field. Operators triaging stuck retry queues have no on-disk record of when retry chain *started* — only `next_attempt_at`. The `\x03` dead-letter row carries `dead_lettered_at` but `\x04` retry-state row does not have a parallel "started" timestamp. ADR-016 §2.6 makes `tokio::time::sleep` the single timekeeping primitive, so this gap is invisible until production triage. Severity: LOW (deferrable; doesn't block Phase 4.B; W4-FIX-* candidate or Wave 5 incremental enhancement). | P3 | wave-4-phase-4a-r8 (2026-05-04) | ADR-016 §2.5 RetryState struct definition + BC-2.18.001 invariants update |
| TD-W4-INJECTION-VOCAB-001 | Wave 4 Phase 4.A R8 spec-reviewer iter-3 (2026-05-04) — SR-LOW-002: S-4.05 + S-4.08 reference `_safety_flags: Vec<String>` populated by InjectionScanner (BC-2.09.004) with "flag, don't strip" semantics, but neither story specifies the canonical flag *name set* the scanner emits (e.g. `"prompt-injection"`, `"sql-injection"`, `"shell-meta"`). Downstream consumers (audit log readers, future SIEM rules on `_safety_flags`) need a documented vocabulary. S-1.10 (InjectionScanner) is the canonical owner; this is a documentation forward-reference, not a contract defect. Severity: LOW (deferrable). | P3 | wave-4-phase-4a-r8 (2026-05-04) | S-4.05 + S-4.08 dev-notes cross-reference to S-1.10 flag-name taxonomy table |
| ~~TD-W4-CV-LOW-001~~ | **RESOLVED 2026-05-04 (D-218 closure).** Wave 4 Phase 4.A R8 consistency-validator iter-3 (2026-05-04) — CV-LOW-001: STORY-INDEX.md lines 25 + 115 cite `BC-INDEX.md v4.27`. Actual BC-INDEX frontmatter version is `v4.32`. The 5-version gap (v4.28 through v4.32) reflects Wave 4 Phase 4.A BC body/title fixes only (BC-2.18.001/002/003/004/008 ActionEngine→ActionDeliveryEngine cleanup + BC-2.12.004 date corrections). Zero BCs added or retired; `active_contracts: 222` and `total_contracts: 230` in BC-INDEX header unchanged and consistent with STORY-INDEX `total_active_bcs: 222`. No structural traceability impact. Severity: LOW (cosmetic version-pin staleness only). | P3 | wave-4-phase-4a-r8 (2026-05-04) | RESOLVED — STORY-INDEX v2.04 BC-INDEX cite updated v4.27→v4.32 (story-writer D-218 burst 2026-05-04) |
| ~~TD-W4-CV-LOW-002~~ | **RESOLVED 2026-05-04 (D-218 closure).** Wave 4 Phase 4.A R8 consistency-validator iter-3 (2026-05-04) — CV-LOW-002: ARCH-INDEX.md line 83 lists ADR-016 v0.14 with date `2026-05-04` (registry-update date). ADR-016 file frontmatter has `date: 2026-05-03` (original authoring date, corrected by Pass-17 F-P17-M-001). Registry-registration-date vs authoring-date discrepancy — well-understood pattern in this project (all 5 other W4 ADRs registered on their authoring date; ADR-016 registry entry was updated in Pass-27 alongside the v0.14 bump). No semantic inconsistency. Severity: LOW (cosmetic date metadata only). | P3 | wave-4-phase-4a-r8 (2026-05-04) | RESOLVED — ARCH-INDEX v2.29 ADR-016 date updated to 2026-05-02 (architect D-218 burst 2026-05-04) |
| TD-HOLDOUT-W1-BACKFILL-001 | Wave 4 Phase 4.A R9 gate (2026-05-04) — D-219 (user catch): Wave 1 (15 stories merged) was implemented BEFORE the formal holdout-gate protocol existed. Zero holdout-evaluation evidence for W1 BCs. Risk: behavioral defects in merged W1 implementations (15 stories, 69 raw BCs) may have shipped without acceptance-scenario verification. Severity: MEDIUM (retroactive risk; not a regression but a gap in convergence evidence). Recommendation: optionally backfill W1 holdout scenarios (foundation HS-001/002/004/005/006/008 cover scenarios but lack BC anchoring) and run gate-step-f-holdout against W1 implementations to detect any latent behavioral defects. | P2 | wave-4-phase-4a-r9-gate (2026-05-04) | author HS-101..HS-105 (W1-specific scenarios) + run holdout-evaluator against W1 merged code |
| TD-HOLDOUT-W2-RETROFIT-001 | Wave 4 Phase 4.A R9 gate (2026-05-04) — D-219 (user catch): Wave 2 (8 stories merged) graded `CONDITIONAL_PASS` at 0.65 mean satisfaction with `11/19 strict / 0.58 partial` must-pass ratio (per `wave_2_gate_step_f_holdout_evaluation` in STATE.md). The weak grade likely reflects coverage gaps in the foundation-only HS-001..HS-008 scenarios (which have NO W2-specific BC anchoring). Investigate retroactively whether the 0.65 grade indicates undiscovered behavioral defects in merged W2 implementations (sensor adapters, RocksDB init, audit construction). Severity: MEDIUM (potential latent defects in shipped code; not a regression but evidence gap). Recommendation: author W2-specific HS files with explicit BC anchoring (W3 HS-003 + HS-007 model) and re-evaluate W2 implementations. | P2 | wave-4-phase-4a-r9-gate (2026-05-04) | author HS-201..HS-204 (W2-specific scenarios with BC-2.01-2.11 anchoring) + run holdout-evaluator against W2 merged code |
| TD-VSDD-054 | Wave 4 Phase 4.B R10 pre-dispatch dep check (2026-05-04) — Pre-Phase-N-implementation dispatch check: verify all `depends_on` chains terminate at `status: merged` stories BEFORE running adversarial spec convergence on Phase N. Discovered: R10 dispatch attempt for S-4.01 found S-4.01 depends on S-3.02 (status=draft); 31 Phase 4.A adversarial passes never checked dep status of S-3.02. The convergence rubric verified internal consistency of W4 specs but did not verify that the implementation dependencies (stories from prior waves) were actually implemented. All 8 W4 stories are transitively blocked. Severity: HIGH (methodological — caused 31-pass wasted spec convergence cycle on unimplementable stories). Codify in: (1) adversary prompt for Phase N.A — add dep-chain check as explicit cross-cut axis; (2) consistency-validator — add `depends_on` terminus check (all terminal deps must be status: merged); (3) wave-kickoff pre-flight checklist — add mandatory dep-status check before Phase N.A dispatch. Discovered: 2026-05-04 by orchestrator pre-R10 dep check after D-218+D-216 closure. | P1 | wave-4-phase-4b-r10-predispatch (2026-05-04) | vsdd-factory adversary prompt + consistency-validator + wave-kickoff pre-flight checklist |
| TD-VSDD-055 | D-226 S-3.01 keystone implementation cycle (2026-05-05) — Per-keystone-story type-design audit before merge. For Tier-1 entry stories that downstream stories depend on, run dclaude:type-design-analyzer audit BEFORE PR creation. S-3.01 audit caught 16 P0/P1 + 3 deviations (untyped FuncCall, missing operators, missing literals, missing Visitor, etc.) that would have propagated to all 12 W3 stories + 8 W4 stories. Codify in per-story-delivery.md: after implementer TDD phase, before demo recording, dispatch type-design-analyzer when story is keystone (Tier-1 position OR direct_dependents >= 5 OR explicitly marked keystone_audit_required: true). Trigger: story frontmatter `tier: 1` OR `keystone: true`. Evidence: 16 P0/P1 + 3 deviations found and fixed BEFORE downstream stories could inherit flawed AST. | P2 | D-226 wave-3-core-s301 (2026-05-05) | vsdd-factory per-story-delivery.md + story template (add keystone_audit_required field) |
| TD-VSDD-056 | D-227 vsdd-factory plugin rc.11 upgrade (2026-05-05) — factory-dispatcher PreToolUse hook tier-3 blocks on certain sub-agent dispatches (github-ops merge commands, certain write operations). Currently shows generic block_intent=true without specifying WHICH tier matched or WHY the block was triggered (which policy rule, what the tool call contained). Makes it difficult to diagnose whether a block is intentional or over-aggressive. Recommendation: surface the tier number + matched rule identifier + the specific argument/context substring that triggered the block. Format: `[factory-dispatcher] BLOCKED tier-N: rule=<rule-id> matched='<excerpt>' tool=<tool-name>`. | P3 | D-227 plugin-rc11-upgrade (2026-05-05) | vsdd-factory factory-dispatcher hook: block message format + diagnostic output |
| TD-VSDD-057 | PR #127 adversary pass-13 F-PG-001 (2026-05-06) — Positive-coverage assertion rule for security-critical CI jobs. Any CI job whose value is regression detection (compile-fail, lint, fuzz-smoke) MUST emit a positive-coverage assertion in its log on every successful run, not just an exit code. Discovered: perimeter-compile-fail Python parser regex was functionally untested for 12+ adversary passes because ANSI color codes in cargo 1.85+ stderr caused `re.match(r'error\[E0603|E0624\]...')` to produce empty `found_names` sets on every run. The job's binary signal (non-zero exit on full-pub regression) was intact, but the per-symbol assertion added at pass-7 F-HIGH-001 was silently bypassed. Fix landed in 9557b647 (`--color=never`). Pattern: log "Perimeter check passed: all N symbols produced E0603/E0624" not just "exit 0". Codify as adversary-prompt rule and/or pre-commit lint for security-critical CI jobs. **Status: OPEN-DEFERRED-CROSS-REPO — targets vsdd-factory plugin repo (separate vsdd-factory session; user has separate prompt targeting plugin repo).** | P2 (OPEN-DEFERRED-CROSS-REPO) | PR-127-pass-13-F-PG-001 (2026-05-06); deferred D-246 (2026-05-06) | vsdd-factory adversary-prompt security-CI-job rule + policy registry codification |
| ~~TD-VSDD-058~~ | **RESOLVED 2026-05-06 (PR #128, squash commit 3e858f9f on develop).** PR #127 adversary pass-14 lens 7 sibling-layer check (2026-05-06) — fuzz-vp021-nightly tight-margin advisory under opt-level=3 (post-30f6fc07 dev-profile crypto opt). Non-blocking for PR #127 but flag for future maintenance: at current 45-min ceiling with ~15 min cold sanitizer build (opt-level=3) + 30 min fuzz run + cleanup, margin is tight. If a future nightly fails in the build phase, bump to 60 min or more. Discovery: fuzz-smoke was silently timing out for 12+ commits since fcc1838c era (run 25427035534 killed at 12m51s in linker phase); fuzz-vp021-nightly has the same root cause exposure but a 45-min ceiling instead of the original 12-min. **Resolution:** maintenance/td-vsdd-058-fuzz-nightly branch, PR #128 — (1) protoc install step added (mirrors all 6 sibling ci.yml jobs, SHA c65c819552d16ad3c9b72d9dfd5ba5237b9c906b); (2) timeout-minutes bumped 45→60 (14 min headroom). All CI checks green. Merged before tonight's 02:00 UTC nightly run. | RESOLVED | PR-127-pass-14-lens7 (2026-05-06); resolved PR-128 (2026-05-06) | CLOSED |

---

### TD-VSDD-047 — CF Key Format Fixes Must Audit All Architecture Docs for the Same CF Name in Lockstep

**Filed:** 2026-05-03 (Wave 4 Phase 4.A Pass 22 F-P22-H-001)
**Severity:** P2 (MEDIUM)
**Source:** Pass 22 discovered that actions.md §"Delivery state" action_state CF key table had 4 stale rows (no `{org_id}:` prefix; `{alert_id}` instead of `{idempotency_key}` on retry row; pre-ADR-016 §2.5 5-row form) even after Pre-Pass-21 broad sweep and Pass 21 fix to data-layer.md. The Pre-Pass-21 sweep fixed actions.md surface-level claims (ActionEngine→ActionDeliveryEngine, 16-permit→8-permit, 1s→60s) but did not audit the action_state CF key table against ADR-016 §2.5 canonical form.

**Gap:** No pre-pass sweep class covers "CF key format table audit." When a CF key is canonicalized in an ADR (e.g., ADR-016 §2.5 action_state key format), the sweep only updates the ADR and the primary data-layer doc. Sister architecture docs that also contain key-format tables for the same CF (e.g., actions.md §"Delivery state") are not in scope.

**Defects surfaced:**
- F-P22-H-001: actions.md action_state CF key table 4-row stale form (no `{org_id}:` prefix; missing discriminator rows \x01–\x05).
- F-P22-M-001: retry row sort-key `{alert_id}` instead of `{idempotency_key}` per ADR-016 §2.5 (subsumed by H-001).

**Recommended additions to pre-pass sweep methodology:**
1. Add sweep class: **CF key format audit** — for every CF named in a finding or ADR change, grep all `architecture/*.md` files for that CF name and audit every key-format table found. Verify canonical form matches the ADR §2.5 (or equivalent) specification.
2. Extend the sweep pattern to include BC files that contain CF key format specifications (e.g., BC-2.18.001).
3. Hook recommendation: validate-cf-key-format.sh — grep for common CF key component tokens and flag non-canonical patterns before commit.

---

### TD-VSDD-048 — Broad-Sweep Methodology Must Include Exhaustive Grep-Completeness Check

**Filed:** 2026-05-04 (Wave 4 Phase 4.A Pass 23 F-P23-L-001 process-gap codification)
**Severity:** P2 (MEDIUM)
**Source:** Pass 23 discovered that `operational-pipeline.md` contained 3 stale references (16-permit, Action Engine, 1-second tick) that survived the Pre-Pass-21 hand-curated broad-sweep target list. The target list for Pre-Pass-21 included: `actions.md`, `module-decomposition.md`, `api-surface.md`, `data-layer.md`, `verification-architecture.md` — but excluded `operational-pipeline.md`. No mechanical check confirmed that all architecture/*.md files containing the canonical stale tokens were in scope.

**Gap:** Every broad-sweep dispatch relies on a hand-curated target list assembled by the orchestrator or state-manager at dispatch time. There is no automated step that:
1. Greps all `specs/architecture/*.md` for canonical stale tokens before the sweep
2. Verifies the resulting hit list is a subset of the sweep target list
3. Fails (aborts dispatch) if any file containing a stale token was not included in the target list

**Defects surfaced:**
- F-P23-H-001: `operational-pipeline.md` had `16-permit` (should be `8-permit` per D-209), `Action Engine` (should be `ActionDeliveryEngine`), and `1-second tick` (should be `60s` per ADR-013 §2.1) — all surviving 2 prior pre-pass sweeps.
- F-P23-M-001: `operational-pipeline.md` changelog had no Wave 4 entries despite Wave 4 architectural changes being reflected in the doc.

**Recommended additions to pre-pass sweep methodology:**

1. **Token-first completeness check (mandatory pre-sweep step):** Before assembling the target list, grep ALL `specs/architecture/*.md` files for the canonical stale tokens relevant to the current sweep context. The grep results define the mandatory minimum target list. Any file producing a hit MUST be in scope.

2. **Canonical stale token list for D-209/ADR-013/rename sweeps:**
   ```
   16-permit | 16 max | 16 concurrent | Action Engine | 1-second tick | ActionEngine[^a-zA-Z]
   ```

3. **Hook recommendation:** `validate-sweep-completeness.sh` — run at the end of every broad-sweep burst:
   ```bash
   # Abort if any stale token survives in any architecture doc
   grep -r "16-permit\|16 max\|16 concurrent\|Action Engine\|1-second tick\|ActionEngine[^a-zA-Z]" \
     .factory/specs/architecture/*.md && echo "STALE TOKENS REMAIN — FIX BEFORE COMMIT" && exit 1 || true
   ```

4. **Process change:** State-manager and orchestrator must run this grep check BEFORE declaring a broad-sweep burst complete. The old pattern of "I swept the files I thought were relevant" must be replaced by "I grepped for the tokens and swept every file that matched."

**Recommended action:** Codify grep-completeness check in vsdd-factory pre-pass sweep skill checklist before next broad-sweep dispatch. Priority: P2 — the current methodology has failed in 2+ consecutive Pre-Pass sweeps (Pre-Pass-21 and Pre-Pass-22 both had completeness gaps caught in the subsequent adversary pass).

---

### TD-VSDD-049 — Comprehensive PRD §2 BC-table ↔ BC H1 Byte-Equal Sync Check

**Filed:** 2026-05-04 (Wave 4 Phase 4.A Pass 24 F-P24-CRIT-001 process implication)
**Severity:** P2 (HIGH)
**Source:** Pass 24 discovered that PRD §2 line 389 BC-2.18.004 table cell title read "Scheduled Report Queries — try_acquire() on 16-Permit Semaphore" — the superseded pre-D-209 title. The canonical BC H1 (source of truth per POL-9) is "Action Delivery Semaphore — 8-Permit Independent Pool". The pre-Pass-24 TD-VSDD-048 sweep fixed the INV-ACTION-004 root-contract prose in PRD §X but did NOT sweep the PRD §2 BC table column for title drift — that is a distinct drift class.

**Gap:** No pre-pass sweep class covers "PRD §2 BC table cell title ↔ BC file H1 byte-equal check." When a BC is renamed or its H1 changes (e.g., D-209 split 16-permit → 8-permit independent pools), the PRD §2 BC table title column may retain the old title independently from the BC file H1.

**Defects surfaced:**
- F-P24-CRIT-001: PRD §2 line 389 BC-2.18.004 title column stale ("Scheduled Report Queries — try_acquire() on 16-Permit Semaphore"); canonical BC H1 is "Action Delivery Semaphore — 8-Permit Independent Pool" per D-209 + ADR-016 §2.1.

**Sweep result (first run, Pass 24):** Comprehensive check of ALL 200 PRD §2 BC rows vs corresponding BC H1 titles found ONLY BC-2.18.004 drift. 199/200 rows were correct. This is a strong signal that the spec corpus is approaching convergence.

**Recommended additions to pre-pass sweep methodology:**

1. **PRD §2 BC table title audit (mandatory for PRD-affecting and BC-H1-renaming bursts):** Mechanically extract all `| [BC-X.YY.ZZZ](...) | <title> |` rows from PRD §2 BC table. For each row, read the corresponding BC file H1. Compare byte-for-byte. Flag any mismatch.

2. **Trigger conditions:** Run this check:
   - At end of EVERY PRD-affecting burst (any change to prd.md)
   - At end of EVERY BC-H1-renaming burst (any BC version bump that changes H1)
   - As part of pre-pass sweep baseline for each adversary pass

3. **Hook recommendation:**
   ```bash
   # Extract BC IDs and titles from PRD §2 BC table
   # Compare each title to the corresponding BC file H1
   # Abort if any mismatch found
   # (implementation: parse markdown table rows matching | [BC-X.YY.ZZZ] | <title> | pattern)
   ```

4. **Process change:** State-manager and product-owner must run this check after every BC-H1 or PRD-table-affecting burst. The old assumption "PRD table stays in sync automatically" is disproven — PRD §2 BC table is a separate copy of BC titles that requires active sync maintenance.

**Recommended action:** Extend vsdd-factory PRD integrity checks to include BC-table-title sync. Priority: P2 — PRD §2 BC table titles encode architectural claims (semaphore sizes, subsystem names) that must be byte-equal to BC H1 canonical titles. A wrong title here misleads implementers about the actual BC they are implementing.

---

## Item Detail

### TD-VSDD-005 — vsdd-factory:adversary Runtime Tool-Binding Defect

**Severity**: P2 (medium — blocks canonical adversarial discipline; workaround available)
**Status**: OPEN
**Opened**: 2026-04-26
**Owner**: vsdd-factory plugin maintainer (separate session)

**Problem**

The `vsdd-factory:adversary` subagent has a runtime tool-binding defect. The agent definition declares `Tools: Read, Grep, Glob` but at runtime only `Read` is bound. This caused:

- Wave 2 gate Pass 1: adversary ran Read-only; POL-1/2/5/6/7/8/9 not fully verified (filed as TD-W2-PASS1-TOOLING-001)
- Wave 2 gate Pass 2: adversary could not be dispatched; fallback to general-purpose-as-adversary

This is the second vsdd-factory plugin-level defect identified in this session (an earlier Skill-tool-empty-body bug was discovered and its fix-prompt deleted per user request).

**Pattern**

These are accumulating plugin-level defects in the vsdd-factory plugin suite. They reduce the reliability of the automated adversarial loop. Until resolved, the workaround is to dispatch `general-purpose` with the adversary role instructions inline.

**Resolution criteria**

- Identify whether tool binding fails at agent-definition parse time or at skill-invocation time
- Fix the vsdd-factory:adversary skill definition to correctly bind Read + Grep + Glob at runtime
- Verify with a test dispatch before Wave 4 gate begins
- Consider adding a tool-verification preamble to every adversarial pass (Pass 2 adopted this — permanent check)

**Workaround** (immediate): Use `general-purpose` agent with adversary instructions + `tools_available: Read, Grep, Glob, Bash` preamble. Verified working in Wave 2/3 gate passes.

### TD-VSDD-001..004 — Stub-as-Impl Prevention Layers

These four items form a cohesive 4-layer defense against stub-as-impl anti-patterns in the vsdd-factory deliver-story flow:

- **Layer 1 (TD-VSDD-001)**: Anti-precedent guard text in SKILL.md / per-story-delivery.md
- **Layer 2 (TD-VSDD-002)**: Red Gate density check gate (RED_TESTS / TOTAL_NEW_TESTS >= 0.5)
- **Layer 3 (TD-VSDD-003)**: `tdd_mode: strict | facade` story frontmatter field
- **Layer 4 (TD-VSDD-004)**: Mutation testing gate for `tdd_mode: facade` stories in wave-gate skill

All four should be addressed together in a single vsdd-factory plugin maintenance session.

### TD-W2-FIXK-001 — validate-consistency Tautology + BC-TV Field-Exclusion Gaps

Two skill-level gaps in `vsdd-factory:validate-consistency`:

1. **Tautology-detector**: No automated check for `test_BC_*` functions that don't call the corresponding `emit_*` function (the test_BC name implies the emitter is exercised, but it may be absent).
2. **BC-TV field-exclusion checker**: No parser for canonical TV tables looking for field-level exclusion markers (e.g., "Token ID in Entry? = No") to cross-reference with struct definitions and test coverage.

Both gaps contributed to HIGH-001 and HIGH-003 findings surviving until Wave 2 gate Pass 7.

### TD-VSDD-038 — Agent Routing Edge Cases for Sweep Bursts

**Filed:** 2026-05-04 (Wave 4 Phase 4.A Pass 11 F-P11-L-001)
**Severity:** P3
**Source:** Adversary Pass 11 finding F-P11-L-001 — BC-2.18.001 v1.6 changelog author=`state-manager` (line 184) violates STATE.md line 469 routing ("BC body/frontmatter → product-owner"). Same pattern observed in Pass 7 BC-2.12.004 v1.5 (also state-manager author).

**Process gap:** When a sweep burst touches BC body content as a side-effect of an index/STATE update, state-manager performs the BC edit directly rather than dispatching product-owner. This violates the agent-routing rules but is operationally efficient for line-level sweep edits.

**Resolution options:**
- (a) Update STATE.md routing rules to permit state-manager for sister-row sweep classes
- (b) Add a routing guard hook that requires product-owner dispatch for any BC body edit
- (c) Accept the divergence and document the exception in this register

**Recommended action:** Defer to vsdd-factory plugin maintenance cycle. Out of Wave 4 scope.

---

### TD-VSDD-039 — Proactive Sweep Checklist Gaps: CF-Key Prefix Order + VP Module-Column Cross-Check

**Filed:** 2026-05-03 (Wave 4 Phase 4.A Pass 13 process-gap codification)
**Severity:** P2
**Source:** Adversary Pass 13 findings F-P13-H-001 + F-P13-H-002 — D-214 Component 1 proactive structural sweep MISSED two HIGH-severity defect classes that Pass 13 then surfaced.

**Gap 1 — CF-key prefix-position sweep:** Sweep did not grep for `{type}:{org_id}:` patterns (org_id NOT in first segment). ADR-008 mandates `{org_id}:` first for `reset_for(org_id)` prefix-scan correctness. S-4.02 had `diff:{org_id}:{schedule_id}:prev` (wrong order) surviving the sweep uncaught.

**Gap 2 — VP module-column cross-check:** Sweep did not compare the module/crate column for each VP across VP-INDEX, verification-architecture.md, verification-coverage-matrix.md, anchor-story task body, and anchor-ADR. Any divergence is a POL-9 violation. VP-053 had `prism-core` in verification-architecture.md while all other sources said `prism-operations`.

**Resolution options:**
- (a) Add CF-key prefix-position grep to standard D-214-class sweep checklist: flag any match where org_id is not the first CF-key segment.
- (b) Add VP module-column cross-check step: for each VP in VP-INDEX, confirm module column is identical in verification-architecture.md and verification-coverage-matrix.md.

**Recommended action:** Codify both checks into vsdd-factory sweep skill checklist before next proactive sweep dispatch.

---

---

### TD-VSDD-040 — state-manager Two-Commit-Protocol Chain-Corruption Recurring Pattern

**Filed:** 2026-05-03 (Wave 4 Phase 4.A Pass 14 process-gap codification — 3rd occurrence)
**Severity:** P2
**Source:** Pass 3 + Pass 5 + pre-Pass-14 chain-corruption episodes. Root cause: `verify-sha-currency.sh` hook re-fires after each SHA-fix commit, detecting the old placeholder SHA and demanding another fix commit, creating an infinite-fixup chain.

**Pattern:** Stage 1 commit contains placeholder SHA `15fa97e6`. Stage 2 replaces placeholder with real Stage 1 SHA. If Stage 2 contains any residual reference to the OLD placeholder (e.g., from a partially-swept file), the hook fires again and demands a 3rd commit — which is explicitly forbidden by the two-commit protocol.

**Suggested fix options:**
- (a) Atomic Stage 2 with empty-SHA-then-amend: write literal `15fa97e6` in Stage 1 for all SHA citation fields; in Stage 2, amend in-place with `git commit --amend` after all files are updated (no 3rd commit created).
- (b) Hook suppression mid-burst: `SKIP_SHA_CHECK=1` for the Stage 2 amend operation only; re-enable immediately after. Documents the suppression in commit message.

**Recommended action:** Defer to vsdd-factory plugin maintenance cycle. Out of Wave 4 scope.

---

### TD-VSDD-041 — Pre-Pass Sweep Missing Audit-Event-Terminology Cross-Check

**Filed:** 2026-05-03 (Wave 4 Phase 4.A Pass 14 F-P14-H-001 process-gap codification)
**Severity:** P2
**Source:** F-P14-H-001 — `ScheduleFireSkipped` vs `ScheduleFireMissed{miss_reason: SemaphoreExhausted}` in S-4.01. The pre-Pass-14 sweep (TD-VSDD-039 codified methodology) did NOT include audit-event-terminology cross-checking and therefore missed this HIGH finding.

**Gap:** Standard sweep checks CF-key prefix order and VP module-column cross-check but has no step for: (1) grep ADR §X.Y Event Taxonomy / audit event sections for declared event token names; (2) grep story Task body + EC emit-call literals; (3) flag mismatch as HIGH candidate before adversary dispatch.

**F-P14-H-001 would have been caught** by this check: ADR-013 §2.4 declares `ScheduleFireMissed { miss_reason: SemaphoreExhausted }`; S-4.01 Task 5 used `ScheduleFireSkipped` — a grep of both would have flagged the mismatch pre-dispatch.

**Recommended action:** Extend standard pre-pass sweep checklist with audit-event-name cross-checking step before Pass 16+.

---

### TD-VSDD-042 — STORY-INDEX Top-Level Aggregates Not in Standard POLICY 9 Cascade Checklist

**Filed:** 2026-05-03 (Wave 4 Phase 4.A Pass 15 F-P15-H-002 process-gap codification)
**Severity:** P2
**Source:** F-P15-H-002 — STORY-INDEX.md `total_vps_assigned` frontmatter field and matching prose overview VP breakdown bullet drifted for 14 passes after Wave 4 ADR-burst added VP-137..145 (9 VPs). VP-INDEX, verification-architecture, and verification-coverage-matrix were all updated correctly, but STORY-INDEX aggregates were missed.

**Gap:** Standard POLICY 9 cascade checklist for VP-addition bursts does NOT include: (4) STORY-INDEX.md `total_vps_assigned:` frontmatter field, or (5) STORY-INDEX.md prose overview VP breakdown (count + per-type tallies).

**Hook recommendation:** Extend `validate-vp-consistency.sh` (POLICY 9 lint hook) to verify STORY-INDEX aggregates against VP-INDEX totals.

**Recommended action:** Extend POLICY 9 checklist and hook before next VP-addition burst.

---

### TD-VSDD-043 — ADR Status H2 Sync Requires Structural Enforcement, Not Textual Checklist

**Filed:** 2026-05-03 (Wave 4 Phase 4.A Pass 16 F-P16-M-002 process-gap codification)
**Severity:** P2 (MEDIUM)
**Source:** F-P16-H-002 — ADR-015 and ADR-018 body `## Status` H2 sections drifted from frontmatter `version:` after the Pass 14 cascade bumped both ADRs. TD-VSDD-039 had already codified the ADR Status H2 vs frontmatter sync check as a textual checklist item, yet the defect recurred. Root cause: cascade bursts that bump ADR frontmatter versions as secondary targets (not primary) do not reliably apply the Status H2 sync step from the textual checklist.

**Gap:** TD-VSDD-039 textual checklist is a human-readable reminder. It has no structural enforcement. Any cascade dispatch that bumps ADR frontmatter `version:` must also sweep the corresponding body `## Status` H2 line — but no hook enforces this invariant.

**Hook recommendation:** Write `validate-adr-status-sync.sh` that:
1. For each `.factory/specs/architecture/decisions/ADR-*.md`, grep frontmatter `version:` field value.
2. Grep body `## Status` H2 line for the version string.
3. Error if mismatch detected.
4. Add to pre-commit hooks for `.factory/specs/architecture/decisions/`.

**Recommended action:** Implement `validate-adr-status-sync.sh` and add to pre-commit hook pipeline before next ADR-bumping burst.

---

### TD-VSDD-046 — Foundation Architecture Docs Need Own Propagation Discipline (Sister to TD-VSDD-039..045)

**Filed:** 2026-05-03 (Wave 4 Phase 4.A Pre-Pass-21 broad-sweep F-PreP21-H-001/H-002)
**Severity:** P2 (MEDIUM)
**Source:** Pre-Pass-21 broad-sweep discovered that foundation architecture documents (actions.md, module-decomposition.md, api-surface.md, data-layer.md, verification-architecture.md) had accumulated stale content from the Pass-6 BC rename (ActionEngine→ActionDeliveryEngine) and earlier tick/permit constant updates. These documents are NOT story files and NOT ADRs — they sit in `.factory/specs/architecture/` as foundation references — yet no pre-pass sweep class explicitly covers them.

**Gap:** TD-VSDD-039 codified the pre-pass sweep methodology and TD-VSDD-041 extended it to audit-event terminology. However, none of TD-VSDD-039..045 explicitly includes foundation architecture docs (module-decomposition.md, actions.md, api-surface.md, data-layer.md) in the sweep class definitions. As a result, renames and constant changes that propagate across ADRs, BCs, and stories can skip foundation docs entirely.

**Defects surfaced:**
- F-PreP21-H-001: `actions.md` had `16-permit` budget (should be `8-permit` per ADR-016 §2.3) and `1-second` tick interval (should be `60s` per ADR-013 §2.2); `module-decomposition.md` crate section stale; `api-surface.md` subsystem list stale; `data-layer.md` column-family count stale; `verification-architecture.md` Mermaid diagram node label stale (P13 sister-fix).
- F-PreP21-H-002: `BC-2.18.003` and `BC-2.18.008` body prose still used `ActionEngine` after the Pass-6 rename to `ActionDeliveryEngine` — sister BCs to `BC-2.18.001` (already fixed Pass 20). Sister-BC sweep was not in the standard propagation checklist.

**Recommended additions to pre-pass sweep methodology (TD-VSDD-039 extension):**
1. Add sweep class: **Foundation arch docs** — for every symbol rename or constant change in any ADR, sweep `.factory/specs/architecture/{actions,module-decomposition,api-surface,data-layer,verification-architecture}.md`.
2. Add sweep class: **Sister-BC propagation** — when one BC in a subsystem group (e.g., BC-2.18.001) is updated for a rename, enumerate all sibling BCs in that subsystem (BC-2.18.002..NNN) and verify propagation.
3. Codify both classes in the pre-pass sweep checklist template (`vsdd-factory/templates/pre-pass-sweep-checklist.md` or equivalent).

---

## Changelog

| Date | Change |
|------|--------|
| 2026-05-06T12:00:00Z | v2.6 — D-246 post-merge update. TD-VSDD-057 status → OPEN-DEFERRED-CROSS-REPO (targets vsdd-factory plugin repo; user has separate prompt for separate vsdd-factory session). TD-VSDD-058 already RESOLVED (PR #128 3e858f9f). 43 items total (1 RESOLVED, 1 OPEN-DEFERRED-CROSS-REPO, 41 OPEN). |
| 2026-05-06T00:00:00Z | v2.5 — TD-VSDD-057 added. 41 → 42 items. TD-VSDD-057 (P2): positive-coverage-assertion rule for security-critical CI jobs — perimeter compile-fail Python regex was untested for 12+ passes due to ANSI color codes in cargo 1.85+ stderr; fix in 9557b647; codification candidate for adversary-prompt + policy registry. Source: PR #127 pass-13 F-PG-001. |
| 2026-05-04T20:00:00Z | v2.4 — TD-VSDD-054 added. 38 → 39 items. TD-VSDD-054 (P1 METHODOLOGICAL): pre-phase-N-implementation dispatch check — verify all `depends_on` chains terminate at `status: merged` stories BEFORE running adversarial spec convergence. Discovered 2026-05-04: R10 dispatch attempt for S-4.01 revealed S-4.01 depends on S-3.02 (status=draft); 31 Phase 4.A adversarial passes never checked dep status; D-223 W3-FIRST pivot. |
| 2026-05-04T14:00:00Z | v2.3 — TD-W4-CV-LOW-001 and TD-W4-CV-LOW-002 marked RESOLVED per D-218 closure (2026-05-04). story-writer updated STORY-INDEX v2.04 (BC-INDEX cite v4.27→v4.32; TD-W4-CV-LOW-001). architect updated ARCH-INDEX v2.29 (ADR-016 date 2026-05-02; TD-W4-CV-LOW-002). Both items closed by D-218 wave-doc-refresh burst. |
| 2026-05-04T12:30:00Z | v2.2 — 7 TD items added (user catch — described in session but never filed). 31 → 38 items. TD-VSDD-053 (P0 structural fix for TD-VSDD-044 6x chain-corruption; self-referential HEAD SHA cites in STATE.md/HANDOFF.md create infinite fix-the-fix loops); TD-W4-RETRY-OBS-001 (P3 R8 SR-LOW-001: RetryState missing first_attempted_at); TD-W4-INJECTION-VOCAB-001 (P3 R8 SR-LOW-002: _safety_flags canonical flag-name set not documented); TD-W4-CV-LOW-001 (P3 R8 CV-LOW-001: STORY-INDEX BC-INDEX cite v4.27 stale vs actual v4.32); TD-W4-CV-LOW-002 (P3 R8 CV-LOW-002: ARCH-INDEX ADR-016 registry date vs frontmatter date cosmetic discrepancy); TD-HOLDOUT-W1-BACKFILL-001 (P2 D-219: W1 never holdout-evaluated — retroactive evidence gap); TD-HOLDOUT-W2-RETROFIT-001 (P2 D-219: W2 0.65 CONDITIONAL_PASS — investigate latent behavioral defects). |
| 2026-05-04T12:00:00Z | v2.1 — TD-VSDD-052 added. 30 → 31 items. TD-VSDD-052: pre-dispatch VP scope verification — orchestrator-prompt VP rationale must match VP-INDEX `Property` canonical text before dispatch; Pass 27 F-P27-H-001 trigger (VP-047 mis-attributed to "action delivery dedup correctness" vs canonical "template variable UUID v7 validation"). NEW class: semantic mis-anchor in VP rationale. Sister of TD-VSDD-051. |
| 2026-05-04T00:00:00Z | v2.0 — TD-VSDD-048 added. 26 → 27 items. TD-VSDD-048: broad-sweep methodology must include exhaustive grep-completeness check; Pre-Pass-21 hand-curated target list missed operational-pipeline.md, allowing 3 stale refs to survive to Pass 23; F-P23-L-001 trigger. |
| 2026-05-03T15:00:00Z | v1.9 — TD-VSDD-047 added. 25 → 26 items. TD-VSDD-047: CF key format fixes must grep all architecture docs for the same CF name and audit all key-format tables in lockstep; actions.md §"Delivery state" 4-row stale form survived Pre-Pass-21 sweep + Pass 21 fix; F-P22-H-001 trigger. |
| 2026-05-03T14:00:00Z | v1.8 — TD-VSDD-046 added. 24 → 25 items. TD-VSDD-046: foundation architecture docs (actions.md, module-decomposition.md, api-surface.md, data-layer.md, verification-architecture.md) + sister-BC propagation need explicit sweep classes in pre-pass methodology; F-PreP21-H-001/H-002 triggers. |
| 2026-05-03T13:00:00Z | v1.7 — TD-VSDD-044+045 added. 22 → 24 items. TD-VSDD-044: state-manager Stage 2 must update ALL SHA-cite fields in STATE.md + HANDOFF.md lockstep (pre-Pass-17 cite-repair trigger). TD-VSDD-045: STORY-INDEX VP Assignment Matrix missing W3/W4 VPs (83 VPs absent; structural gap deferred to post-convergence; F-P17-M-002 trigger). |
| 2026-05-03T12:30:00Z | v1.6 — TD-VSDD-043 added. 21 → 22 items. TD-VSDD-043: ADR Status H2 sync requires structural lint-hook enforcement; textual checklist (TD-VSDD-039) insufficient for cascade bursts (F-P16-M-002 trigger). |
| 2026-05-03T12:15:00Z | v1.5 — TD-VSDD-042 narrative section added (was table-only). 20 items unchanged. |
| 2026-05-03T12:00:00Z | v1.4 — TD-VSDD-040+041 added. 18 → 20 items. TD-VSDD-040: two-commit-protocol chain-corruption 3rd recurrence (Pass 3+5+pre-P14). TD-VSDD-041: pre-pass sweep missing audit-event-terminology class (F-P14-H-001 trigger). |
| 2026-05-03T00:00:00Z | v1.3 — TD-VSDD-039 added. 17 → 18 items. Filed per Pass 13 process-gap codification: proactive sweep missed CF-key-prefix-order and VP-module-column-drift defect classes. |
| 2026-05-02 | v1.0 — Initial creation. 13 items carved out from `.factory/tech-debt-register.md` per user directive. Items moved: TD-VSDD-001/002/003/004/005, TD-W2-PASS1-TOOLING-001, TD-VSDD-029/030/031/032/033/034, TD-W2-FIXK-001. |
| 2026-05-02T12:00:00Z | v1.1 — TD-VSDD-035/036/037 added. 13 → 16 items. Filed per user catch: Wave 4 pre-flight cycle-manifest pattern is itself a methodology innovation pending vsdd-factory codification. |
| 2026-05-04T00:30:00Z | v1.2 — TD-VSDD-038 added. 16 → 17 items. Filed per Pass 11 F-P11-L-001: agent routing process-gap for sweep bursts that touch BC body content. |
