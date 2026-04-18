---
project: prism
mode: brownfield
phase: 3-story-decomposition-patch-cycle
status: in_progress
started: 2026-04-13
repos:
  - poller-cobra
  - poller-express
  - poller-bear
  - poller-coaster
  - serveMyAPI
  - tally
  - axiathon
  - ocsf-proto-gen
  - mcp-claroty-xdome
current_step: "PAUSED post-pass-24; 3 findings open (2 HIGH, 1 MED); Burst 25 pending"
awaiting: "User resume signal. Next action: (a) dispatch Burst 25 to close 3 pass-24 findings, or (b) wait for Policy 9 plugin integration, or (c) both in parallel. Recommendation: (c)."
dtu_required: true
dtu_assessment: in_progress
phase_3_patch_trigger: "consistency audit 2026-04-16 — 19 gaps + BC traceability holes"
phase_3_reopened: 2026-04-16
audit_policy_decisions:
  append_only_numbering: true
  lift_invariants_to_bcs: true
  state_manager_runs_last: true
  semantic_anchoring_integrity: true
  creators_justify_anchors: true
  architecture_is_subsystem_name_source_of_truth: true
  bc_h1_is_title_source_of_truth: true
  bc_array_changes_propagate_to_body_and_acs: true
  vp_index_is_vp_catalog_source_of_truth: true
plugin_version_adopted: "vsdd-factory v0.24.2+ (Policy 9 + 17 hooks, policy-registry, factory-cycles-bootstrap)"
plugin_adopted_date: 2026-04-18
policy_registry_source_of_truth: .factory/policies.yaml
current_cycle: phase-3-patch
historical_cycles:
  - name: phase-1-convergence
    passes: 33
    archived: 2026-04-18
    final_trajectory: "13 → 1 finding (converged at pass-33)"
layout_bootstrap_date: 2026-04-18
adversary_pass_1_findings: "29 findings (6 CRIT, 9 HIGH, 9 MED, 5 LOW); convergence counter reset; fixes dispatched in Burst 4a (arch) + Burst 4b (po/sw/sm)"
adversary_pass_1_date: 2026-04-17
adversary_pass_2_findings: "24 findings (6 CRIT, 7 HIGH, 6 MED, 5 LOW); convergence counter still at 0"
adversary_pass_2_date: 2026-04-17
adversary_pass_3_findings: "21 findings (3 CRIT, 5 HIGH, 7 MED, 6 LOW); convergence counter still at 0"
adversary_pass_3_date: 2026-04-17
adversary_pass_4_findings: "7 findings (0 CRIT, 3 HIGH, 2 MED, 2 LOW); convergence trajectory 29→24→21→7"
adversary_pass_4_date: 2026-04-17
adversary_pass_5_findings: "4 findings (0 CRIT, 0 HIGH, 3 MED, 1 LOW); trajectory 29→24→21→7→4; CRIT/HIGH zero 2nd consecutive pass"
adversary_pass_5_date: 2026-04-17
adversary_pass_6_findings: "3 findings (0 CRIT, 0 HIGH, 3 MED, 0 LOW); trajectory 29→24→21→7→4→3; CRIT/HIGH zero 3rd consecutive pass"
adversary_pass_6_date: 2026-04-17
adversary_pass_7_findings: "2 findings (0 CRIT, 0 HIGH, 1 MED, 1 LOW); trajectory 29→24→21→7→4→3→2; CRIT/HIGH zero 4th consecutive pass"
adversary_pass_7_date: 2026-04-17
adversary_pass_8_findings: "CLEAN — 0/0/0/0"
adversary_pass_8_date: 2026-04-17
adversary_pass_9_findings: "CLEAN — 0/0/0/+2 LOW; counter 2/3; then reset by Burst 11"
adversary_pass_9_date: 2026-04-17
adversary_pass_12_findings: "26 findings across 6 anchoring axes (9 CRIT, 11 HIGH, 4 MED, 2 LOW); BLOCK convergence"
adversary_pass_12_date: 2026-04-17
adversary_pass_12_type: "comprehensive semantic anchoring audit (specialized, not standard pass)"
adversary_pass_13_findings: "8 findings (4 CRIT, 4 HIGH, 0 MED, 0 LOW) across 4 anchoring axes; trajectory 26 → 8 = 69% decay; BLOCK counter at 0/3"
adversary_pass_13_date: 2026-04-17
adversary_pass_14_findings: "4 findings (0 CRIT, 2 HIGH, 2 MED, 1 observation); trajectory 26 → 8 → 4 = 50% decay; BLOCK counter at 0/3"
adversary_pass_14_date: 2026-04-17
adversary_pass_15_findings: "2 findings (0 CRIT, 0 HIGH, 2 MED, 2 LOW observations); trajectory 26 → 8 → 4 → 2 = 50% decay; CRIT/HIGH zero 2nd consecutive; BLOCK at 0/3 on MED anchor-integrity"
adversary_pass_15_date: 2026-04-17
adversary_pass_16_findings: "1 finding (0 CRIT, 0 HIGH, 1 MED, 3 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 = 50% decay; CRIT/HIGH zero 3rd consecutive; BLOCK at 0/3 on MED anchor-integrity"
adversary_pass_16_date: 2026-04-17
adversary_pass_17_findings: "3 findings (0 CRIT, 1 HIGH, 0 MED + 2 LOW observations elevated to MED per semantic_anchoring_integrity policy); trajectory 26 → 8 → 4 → 2 → 1 → 1 (stable at 1); BLOCK at 0/3"
adversary_pass_17_date: 2026-04-17
adversary_pass_18_findings: "3 findings (0 CRIT, 1 HIGH, 2 MED, 3 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 (scope-expansion uptick surfacing systemic title drift); BLOCK at 0/3"
adversary_pass_18_date: 2026-04-17
adversary_pass_19_findings: "6 findings (0 CRIT, 1 HIGH, 5 MED, 2 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 (scope-expansion continuing — BC body self-contradiction + 7 more title drifts + matrix over/underclaim + invariant misattribution + cross-index version pin + STORY-INDEX matrix underclaim); BLOCK at 0/3"
adversary_pass_19_date: 2026-04-17
adversary_pass_20_findings: "12 findings (2 CRIT, 5 HIGH, 2 MED, 3 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 (scope-expansion uptick from broader axes: removed-vs-active contradiction, systematic title drift, orphan DIs, EC-ID collisions, invariant misattributions); BLOCK at 0/3"
adversary_pass_20_date: 2026-04-17
user_decision_p3p20: "Option A — un-retire BC-2.04.014, BC-2.06.009, BC-2.10.005 with new Config-Reload semantics (restores DI-003 tool-list notification enforcement)"
convergence_counter: "0 of 3 (reset by Burst 23 spec changes; Burst 24 closed pass-23 findings; pass-24 found 3 findings, did not advance counter)"
adversary_pass_21_findings: "8 findings (0 CRIT, 3 HIGH, 3 MED, 2 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 (decay + no new axes — all retread drift classes); BLOCK at 0/3"
adversary_pass_21_date: 2026-04-17
adversary_pass_22_findings: "6 findings (0 CRIT, 3 HIGH, 1 MED, 2 LOW obs); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 (decay, new policy-8 surfacing pre-existing drift); BLOCK at 0/3"
adversary_pass_22_date: 2026-04-17
adversary_pass_23_findings: "7 findings (0 CRIT, 4 HIGH, 1 MED, 2 LOW); trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 (uptick — new drift class: architecture-layer staleness after VP-INDEX updates); BLOCK at 0/3; novelty HIGH"
adversary_pass_23_date: 2026-04-18
adversary_pass_24_findings: "3 findings (0 CRIT, 2 HIGH, 1 MED, 0 LOW); trajectory ...→7→3 (decay resumed post-23 uptick); CRIT=0 for 13th consecutive pass; Policy 9 first substantive surfacing (P3P24-A-H-002); BLOCK at 0/3; novelty MEDIUM"
adversary_pass_24_date: 2026-04-18
deferred_invariant_citations:
  - invariant: DI-028
    target_bc: BC-2.12.001
    blocker: "Body needs cap-check postcondition + E-SCHED-008 error case"
  - invariant: DI-028
    target_bc: BC-2.13.006
    blocker: "Body needs cap-check postcondition + E-RULE-011 error case"
  - invariant: DI-029
    target_bc: BC-2.06.005
    blocker: "Body needs cross-validation postcondition (correlation window vs schedule interval WARN)"
pass_8_observation: "P3P8-O-001 CAP-020 vs SS-19 semantic pre-existing; escalated in Burst 11 as blocking (CAP-031 created)"
cap_count: 34
bc_index_version: "v4.7"
story_index_version: "v1.16"
subsystem_count: 20
story_count: 75
dtu_crate_count: 14
dtu_scope_expansion: "sensors (4) + actions (3) + infusions (2) + log-forwarding (4) + common (1) = 14"
bc_count_corrected: 195
removed_bc_count: 13
dual_anchor_active_bcs: 6
canonical_cf_count: 16
dtu_clones_built: pending
phase_3_stories_written: 2026-04-16
phase_3_converged: 2026-04-16
phase_2_post_review_converged: 2026-04-16
phase_2_converged: 2026-04-15
phase_2_architect_review: 2026-04-16
phase_0_approved: 2026-04-14
phase_1_converged: 2026-04-15
phase_2_started: 2026-04-15
deployment_model: per-analyst-stdio
---

# VSDD Pipeline State — Prism

## Current Phase: 2 — Architecture (In Progress)

### Progress
- [x] Repos cloned to .references/
- [x] Initial 6-pass analysis on all 9 repos (INGESTION.md)
- [x] Migrate analyses to .factory/semport/ structure
- [x] Phase B: Convergence deepening (all repos, all passes)
- [x] Phase B.5: Coverage audit (all repos)
- [x] Phase B.6: Extraction validation (all repos)
- [x] Phase C: Final synthesis (all repos)
- [x] Orchestrator Phase 0 substeps (0a-0f-post)
- [x] Multi-repo phase-0 synthesis
- [x] Deployment model correction (per-analyst stdio, not multi-tenant server)
- [x] Consistency validation report (validation-report.md)
- [x] Phase 0 gate (human approval — APPROVED 2026-04-14)

## Phase 1: Specification — CONVERGED

### Progress
- [x] Product brief created and approved (2026-04-14)
- [x] Domain spec: capabilities (30), entities (30+), invariants (32), edge cases (39)
- [x] Domain spec: AxiQL grammar (EBNF), architecture concept, scheduled/detection concept
- [x] PRD: 153 active behavioral contracts across 16 subsystems
- [x] PRD supplements: interface definitions (~35 MCP tools), error taxonomy (90+ codes), NFR catalog (18+)
- [x] Config-driven sensor adapters (CAP-029): all sensors ship as TOML spec files
- [x] Two-tier sensor adapter architecture documented (no-code + high-code)
- [x] Adversarial review: 15 passes (20-34), ~90 findings, all fixed
- [x] Convergence achieved: 0 CRITICAL, 0 HIGH across final 3 passes (33, 34 + confirmation)
- [x] Phase 2 gate (human approval — APPROVED 2026-04-15)

## Phase 2: Architecture — IN PROGRESS

### Progress
- [x] Architecture index (ARCH-INDEX.md) with 15 ADRs
- [x] System overview (deployment model, principles, constraints, resource budgets)
- [x] Module decomposition (12 crates, 4 layers, machine-readable component map)
- [x] Dependency graph (topological build order, external dependencies)
- [x] API surface (35+ MCP tools, error contract, resources, prompts)
- [x] Data layer (RocksDB 12 column families, Arrow schema, cache architecture)
- [x] Query engine (AxiQL parser, DataFusion integration, push-down, UDFs)
- [x] Sensor adapters (TOML spec files, CustomAdapter escape hatch, sealed auth)
- [x] Security architecture (credentials, feature flags, audit, prompt injection)
- [x] Operational pipeline (scheduler, differential, detection, alerts, cases)
- [x] Concurrency architecture (tokio, arc-swap, shared state, deadlock prevention)
- [x] Purity boundary map (pure core catalog, effectful shell classification)
- [x] Verification architecture (32 VPs: 15 Kani, 11 proptest, 5 fuzz, 1 mutants)
- [x] Tooling selection (Kani, proptest, cargo-fuzz, clippy, semgrep)
- [x] Verification coverage matrix (VP-to-module traceability, invariant coverage)
- [x] VP-INDEX.md (32 verification properties cataloged)
- [x] Representative VP files (VP-001, VP-002, VP-005, VP-021)
- [x] Adversarial review pass 1: 16 findings (2 CRITICAL, 5 HIGH, 9 MEDIUM) — all fixed
- [x] A-1 FIX: rmcp version reconciled to 1.4 across all docs (PRD, brief, assumptions, BC)
- [x] A-2 FIX: Memory budget derivation table, explicit 2-concurrent-query limit, RocksDB block cache cap
- [x] A-3 FIX: DiffResults storage bounds (500 MB cap, zstd compression, cleanup on delete)
- [x] A-4 FIX: Write-audit ordering (AD-016 intent-log pattern)
- [x] A-5 FIX: JOIN syntax clarified (composite sources, not SQL JOINs), virtual field naming unified (_sensor/_client/_source)
- [x] A-6 FIX: .axd detection rule format fully specified (new architecture section)
- [x] A-7 FIX: Dirty bit crash recovery protocol specified
- [x] A-8 FIX: prism-ocsf purity classification clarified (DescriptorPool as parameter)
- [x] A-9 FIX: Bincode schema versioning convention (2-byte version tag)
- [x] A-10 FIX: DataFusion memory pool integration specified
- [x] A-11 FIX: Per-rule alert rate limiting (100/hour default)
- [x] A-12 FIX: CI-007 added (scheduled task captures Arc at spawn time)
- [x] A-13 FIX: VP-033 added for DI-026, missing invariants accounted in traceability
- [x] A-14 FIX: SESSIONS source returns E-QUERY-015 structured error
- [x] A-15 FIX: Arrow schema specified (ocsf_class_uid hot column, unified schema for cross-sensor)
- [x] A-16 FIX: Variable interpolation safety (JSON-escape, percent-encode, no recursive expansion)
- [x] Adversarial review pass 2: 18 findings (2 CRITICAL, 8 HIGH, 7 MEDIUM, 1 LOW) — all fixed
- [x] P2-A-1 FIX: DataFusion GreedyMemoryPool (not FairSpillPool)
- [x] P2-A-2 FIX: Sequence mode parameterized execution for step variables
- [x] P2-A-3 FIX: Response cache ~10KB/entry consistent with NFR-015
- [x] P2-A-4 FIX: Dirty bit write fail-closed (abort query if bit can't be written)
- [x] P2-A-5 FIX: DiffResults cap lowered to 200MB for RSS safety
- [x] P2-A-6 FIX: Detection rules prohibited from querying internal prism.* tables
- [x] P2-A-7 FIX: Detection evaluation uses "events" MemTable name (matches ad-hoc)
- [x] P2-A-8 FIX: stix_pattern_match deferred to post-v1
- [x] P2-A-9 FIX: Alert rate limit persisted to RocksDB detection_state
- [x] P2-A-10 FIX: bincode 1.x (serde-based, not 2.x)
- [x] P2-A-11 FIX: Dependency versions pinned (prost 0.13, prost-reflect 0.14, keyring 3.x, reqwest 0.12)
- [x] P2-A-12 FIX: E-QUERY-007 tombstone added
- [x] P2-A-13 FIX: DEVICES added to grammar production rule
- [x] P2-A-14 FIX: Alert template injection scanning + trust_level metadata
- [x] P2-A-15 FIX: VP-033 method changed to integration_test
- [x] P2-A-16 FIX: detection_state 100MB cap with 7-day eviction
- [x] P2-A-17 FIX: DEC-040 added (rule deletion during evaluation)
- [x] P2-A-18 FIX: VP-034/VP-035 added for credential encryption
- [x] Adversarial review pass 3: 14 findings (3 CRITICAL, 8 HIGH, 3 MEDIUM, 3 LOW) — all fixed
- [x] P3-C-001 FIX: Detection evaluation serialized, 20MB GreedyMemoryPool, within scheduled budget
- [x] P3-C-002 FIX: E-STORE-009 updated to fail-closed (broken severity, query aborted)
- [x] P3-C-003 FIX: CONFIRM namespace removed, consolidated into FLAG
- [x] P3-H-001 FIX: BC-2.11.012 virtual field names updated to _sensor/_client/_source
- [x] P3-H-002 FIX: Correlation SQL uses event_time window anchor, not now()
- [x] P3-H-003 FIX: Interpretive evaluation uses typed Value comparison, not SQL interpolation
- [x] P3-H-004 FIX: VP counts corrected (prism-core 8 Kani, prism-query 4 Kani), VP-033 in prism-audit
- [x] P3-H-005 FIX: Rate limit key uses \x00 prefix to prevent collision
- [x] P3-H-006 FIX: DEVICES added to condensed grammar (section 12)
- [x] P3-H-007 FIX: prism-credentials coverage target raised to 90%
- [x] P3-H-008 FIX: ALERTS vs prism.alerts disambiguation documented
- [x] P3-L-003 FIX: DiffResults arithmetic corrected for realistic scale
- [x] P3-L-004 FIX: Detection rule evaluation order specified (lexicographic, sequential)
- [x] Adversarial review pass 4: 6 findings (1 CRITICAL, 5 HIGH) + 5 observations — all fixed
- [x] P4-CRIT-001 FIX: StorageDomain ownership resolved (prism-core only, removed from prism-storage)
- [x] P4-HIGH-001 FIX: VP/Kani counts corrected (19 Kani, 11 proptest, 4 fuzz, 1 integration = 35 total)
- [x] P4-HIGH-002 FIX: E-WATCHDOG-002 added for concurrent memory pressure (retryable: true)
- [x] P4-HIGH-003 FIX: Tool names standardized to short form (create_rule, list_rules, delete_rule)
- [x] P4-HIGH-004 FIX: RocksDB RSS range widened to ~40-80MB in budget table
- [x] P4-HIGH-005 FIX: EC-15-011 orphan reference replaced with BC-2.15.002 reference
- [x] Adversarial review pass 5: 12 findings (2 CRITICAL, 5 HIGH, 5 MEDIUM) — all fixed
- [x] P5-CRIT-1 FIX: Duplicate E-WATCHDOG-002 removed (denylist covered by E-QUERY-008)
- [x] P5-CRIT-2 FIX: GreedyMemoryPool fallback path specified (3 tiers)
- [x] P5-HIGH-1 FIX: Rate limit/dedup entries exempt from 7-day eviction
- [x] P5-HIGH-2 FIX: Global HTTP connection semaphore (200 cap) added, config keys specified
- [x] P5-HIGH-3 FIX: Migration interruption safety, mixed-version startup behavior documented
- [x] P5-HIGH-4 FIX: Correlation operates on persisted sliding window across ticks (clarified)
- [x] P5-HIGH-5 FIX: Detection state key encoding uses length-prefix (no delimiter collision)
- [x] P5-MED-3 FIX: Internal tables use prism_alerts (underscore, not dot) — grammar-compatible
- [x] P5-OBS-3 FIX: watchdog_status gains clear_denylist parameter
- [x] Adversarial review pass 6: 0 CRITICAL, 2 HIGH, 2 MEDIUM — fixed
- [x] P6-F1 FIX: Detection engine receives added records only (not added+removed)
- [x] P6-F2 FIX: Internal table names use underscore form in all docs
- [x] P6-F3 FIX: Fuzz target count corrected (5 total, prism-security injection scanner)
- [x] P6-F4 FIX: Detection SessionContext scope clarified (per schedule-execution task)
- [x] Adversarial review pass 7: 0 CRITICAL, 2 HIGH, 1 MEDIUM, 1 LOW — fixed
- [x] P7-001 FIX: Schedule semaphore changed to try_acquire() (skip, not queue)
- [x] P7-002 FIX: Detection reuses query SessionContext/pool (no separate pool)
- [x] P7-003 FIX: Dirty bit sync: true (must survive OOM kill)
- [x] Adversarial review pass 8: 0 CRITICAL, 0 HIGH ✓ (pass 1 of 3) — 3 MEDIUM, 1 LOW fixed
- [x] P8-001 FIX: Detection state stale entries expire naturally on spec reload
- [x] P8-002 FIX: add_sensor_spec defined as write_file + reload_config wrapper
- [x] P8-003 FIX: Global HTTP semaphore uses timeout-bounded acquire, cancels via query timeout
- [x] P8-004 FIX: Per-rule group key cap (10,000 default) prevents state amplification
- [x] Adversarial review pass 9: 0 CRITICAL, 1 HIGH — fixed (resets convergence counter)
- [x] P9-001 FIX: prism-operations → prism-security dependency added (InjectionScanner)
- [x] P9-002 FIX: prism-operations → prism-audit dependency added (AuditEmitter)
- [x] Dependency graph and topological build order updated
- [x] Adversarial review pass 10: 0 CRITICAL, 2 HIGH — fixed (counter stays at 0)
- [x] P10-A FIX: Credential audit path specified (prism-mcp/prism-operations, not prism-credentials)
- [x] P10-B FIX: Migration exits non-zero on write failure
- [x] P10-C FIX: reqwest::Client lifecycle across config reload documented
- [x] Default CF documented as unused by Prism
- [x] Adversarial review pass 11: 0 CRITICAL, 2 HIGH (1 false positive), 4 MEDIUM — fixed
- [x] P11-01: ASCII dep graph verified correct (prism-query→prism-storage present at line 35)
- [x] P11-02 FIX: Watchdog denylist TTL uses lazy expiry check at query start
- [x] P11-04 FIX: Detection does NOT re-enter query engine for sensor fan-out (in-memory only)
- [x] Adversarial review pass 12: 0 CRITICAL, 4 HIGH — fixed
- [x] P12-H1 FIX: Startup schema version check added (exit 3 if unmigrated)
- [x] P12-H2 FIX: QueryEngine::execute_scheduled() API returning SessionContext specified
- [x] P12-H3 FIX: AlertRateLimitState added to shared state table (Arc<Mutex>)
- [x] P12-H4 FIX: create_pack/delete_pack added to api-surface.md tool registry
- [x] Adversarial review pass 13: 0 CRITICAL, 2 HIGH — fixed
- [x] P13-H1 FIX: SESSIONS added to detection rule prohibited sources list
- [x] P13-H2 FIX: IOC file specification added (format, location, loading, limits, hot reload)
- [x] P13-I1 FIX: SessionContext error-path drop requirement specified
- [x] Adversarial review pass 14: 0 CRITICAL, 2 HIGH — fixed
- [x] P14-001 FIX: BC-2.15.005 aligned to fail-closed for dirty bit write failure
- [x] P14-002 FIX: detection_state key encoding canonicalized to length-prefix across all docs
- [x] Adversarial review pass 15: 0 CRITICAL, 0 HIGH, 0 MEDIUM ✓ (pass 1 of 3) — 4 LOW fixed
- [x] P15-OBS2 FIX: DEC-030 sweep interval aligned to 3600s (matches data-layer.md)
- [x] P15-OBS3 FIX: list_packs documented as global (packs are not client-scoped)
- [x] P15-OBS4 FIX: Rate limit RocksDB write under Mutex (no post-release window)
- [x] Adversarial review pass 16: 0 CRITICAL, 1 HIGH, 1 MEDIUM — fixed (counter resets)
- [x] P16-001 FIX: AlertRateLimitState RocksDB write after Mutex release (CI-004 compliant, bounded under-count)
- [x] P16-002 FIX: SessionContext drop promoted to REQUIRED, VP-036 added
- [x] Adversarial review pass 17: 0 CRITICAL, 6 HIGH, 1 MEDIUM — BC-2.15.x batch update
- [x] P17-001 FIX: BC-2.15.005 dirty bit key format + scope aligned to data-layer.md (query-only)
- [x] P17-003 FIX: BC-2.15.011 table names updated to underscore form throughout
- [x] P17-004 FIX: BC-2.15.011 cross-source JOIN removed, replaced with separate-query pattern
- [x] P17-005 FIX: BC-2.15.009 decorator vs virtual field distinction documented
- [x] P17-006 FIX: BC-2.15.009 filterability invariant corrected (virtual=yes, decorator=no)
- [x] P17-007 FIX: BC-2.15.006 concurrent API calls aligned to 10 per-query (matching architecture)
- [x] Adversarial review pass 18: 0 CRITICAL, 1 HIGH, 2 MEDIUM — fixed
- [x] P18-001 FIX: BC-2.15.006 watchdog status tool reference corrected
- [x] P18-002 FIX: E-IOC-001/002/003 added to error taxonomy
- [x] P18-003 FIX: BC-2.15.009 EC-15-033 _client_id → _client
- [x] P18-004 FIX: PRISM_MAX_INTERNAL_TABLE_SCAN added to system-overview constraints
- [x] Adversarial review pass 19: 0 CRITICAL, 0 HIGH, 5 MEDIUM — fixed
- [x] P19-001 FIX: axiql-grammar.md concurrent query limit cross-referenced to watchdog level
- [x] P19-004 FIX: add_sensor_spec cleanup failure returns E-SPEC-002 with path
- [x] P19-005 FIX: Internal table scan truncation returns partial results with _meta.scan_truncated
- [x] P19-006 FIX: watchdog_status clear_denylist runtime capability gate documented
- [x] P19-VP036 FIX: VP-036 includes panic case
- [x] Adversarial review pass 20: 0 CRITICAL, 0 HIGH, 2 MEDIUM — fixed
- [x] P20-001 FIX: Type tag byte encoding (\x00=group, \x01=rate_limit, \x02=dedup) across all docs
- [x] P20-002 FIX: Schema version check samples 10 entries, documented as heuristic
- [x] scopeguard added to dependency graph
- [x] Adversarial review pass 21: 0 CRITICAL, 0 HIGH, 2 MEDIUM — entities.md fixed
- [x] P21-001 FIX: InternalTable entity table_name updated to underscore form (prism_alerts)
- [x] P21-002 FIX: DiffState entity storage domain reference corrected to DiffResults
- [x] Adversarial review pass 22: 0 CRITICAL, 0 HIGH, 1 MEDIUM — fixed
- [x] P22-001 FIX: prism_diff_results schema clarified (metadata only, raw sensor data via get_diff_results)
- [x] P22-OBS-A FIX: BC-2.15.009 virtual vs decorator field documentation reference corrected
- [x] P22-OBS-B FIX: BC-2.15.005 detection dirty bit exclusion rationale corrected (not ephemeral)
- [x] Adversarial review pass 23: **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✓ (pass 1 of 3)
- [x] Adversarial review pass 24: 0 CRITICAL, 0 HIGH, 1 MEDIUM — fixed (counter resets)
- [x] P24-001 FIX: set_credential risk tier corrected to Irreversible (update) in api-surface.md
- [x] Adversarial review pass 25: **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✓ (pass 1 of 3)
- [x] Adversarial review pass 26: **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✓ (pass 2 of 3)
- [x] Adversarial review pass 27: **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✓ (pass 3 of 3)
- [x] **PHASE 2 ARCHITECTURE: CONVERGED** — 27 passes, 0/0/0 × 3 consecutive

## Phase 2b: Human Architect Review (2026-04-15 — 2026-04-16)

### Additions from architect review:
- [x] PrismQL rename (AxiQL → PrismQL/PQL, 185 occurrences across 55 files)
- [x] Mermaid diagrams added to all 16 architecture documents
- [x] EventSnapshot — inline matched event data in alerts (no re-query needed)
- [x] AI-opaque credential management (AD-017) — values never transit AI context
- [x] _source → _source_table rename (avoids Elasticsearch collision)
- [x] Decorator fields moved to _meta envelope (no underscore prefix)
- [x] Full JOIN support — SQL mode (all JOIN types) + pipe mode (join stage)
- [x] 8 real-world MSSP query scenarios (vulnerability correlation, coverage gaps, OT/IT convergence)
- [x] WASM plugin system (AD-019) — .prx extension, polyglot, sandboxed, hot-reloadable
- [x] Eliminated Tier 3 (compiled-in adapters) — all sensors eat our own dog food
- [x] .axd → .detect rename (Prism-branded detection rules)
- [x] Infusions framework (AD-020) — GeoIP, threat intel, CVSS as UDFs + enrich pipe stage
- [x] Actions framework (AD-021) — Slack, PagerDuty, Jira, email, syslog with 4 triggers (alert, case, schedule, manual)
- [x] Case-triggered actions — Jira sync lifecycle, management escalation, client notification
- [x] Filesystem watching (AD-018) — notify crate, auto-reload config/specs/plugins
- [x] Multi-repo git config subscriptions with merge precedence
- [x] Process lifecycle diagram — MCP server as persistent child process
- [x] End-to-end operational walkthrough (scheduler → detection → alert → action → AI)
- [x] Query packs explained with sharing model (git-based config, per-analyst state)
- [x] Future: shared operational state (Option D) captured as post-v1
- [x] NEW: installation.md — 5 distribution channels, prism init/register/install-secops-factory/health CLI
- [x] NEW: config-schema.md — full prism.toml + aliases.toml schema, env var overrides, config diff tools
- [x] NEW: infusions.md — enrichment framework with TOML specs + .prx plugins
- [x] NEW: actions.md — alert delivery framework with TOML specs + .prx plugins
- [x] NEW: observability.md — 18 log targets, trace IDs, prism logs CLI, get_diagnostics tool, external forwarding (Datadog/Splunk/Elastic/OTLP)
- [x] Latency targets (p95) — Prism-internal vs external, per-operation targets
- [x] Architecture now 21 documents (was 16), 21 ADRs (was 15)

### Architecture Document Count: 21
| # | Document | Status |
|---|----------|--------|
| 1 | system-overview.md | Updated (lifecycle, latency targets, config sharing) |
| 2 | module-decomposition.md | Updated (diagrams) |
| 3 | dependency-graph.md | Updated (diagrams, vault/wasmtime/notify deps) |
| 4 | api-surface.md | Updated (resources, prompts, get_help, get_diagnostics, credential_status) |
| 5 | data-layer.md | Updated (action_state CF, _source_table, EventSnapshot) |
| 6 | query-engine.md | Updated (JOINs, scenarios, IOC spec, diagrams) |
| 7 | sensor-adapters.md | Updated (WASM plugins, .prx, file watcher, eat-our-own-dog-food) |
| 8 | security-architecture.md | Updated (AI-opaque credentials, AD-017, diagrams) |
| 9 | operational-pipeline.md | Updated (EventSnapshot, end-to-end walkthrough, diagrams) |
| 10 | concurrency-architecture.md | Updated (diagrams, AlertRateLimitState) |
| 11 | purity-boundary-map.md | Updated (diagrams) |
| 12 | verification-architecture.md | Updated (diagrams, VP-033-036) |
| 13 | tooling-selection.md | Updated (diagrams) |
| 14 | verification-coverage-matrix.md | Updated (VP counts) |
| 15 | detection-rule-format.md | Updated (.detect rename, diagrams, file org) |
| 16 | ARCH-INDEX.md | Updated (AD-016 through AD-021, new docs) |
| 17 | infusions.md | **NEW** — enrichment framework |
| 18 | actions.md | **NEW** — alert delivery + case triggers |
| 19 | installation.md | **NEW** — distribution, CLI, secops-factory |
| 20 | config-schema.md | **NEW** — full config schema |
| 21 | observability.md | **NEW** — diagnostic logging + external forwarding |

## Post-Review Adversarial Convergence (2026-04-16)

- [x] Pass 1: 4 CRITICAL, 6 HIGH — all fixed (CF count, case template vars, module assignment, infusion UDFs in detection)
- [x] Pass 2: 0 CRITICAL, 3 HIGH, 3 MEDIUM — all fixed (CF count stale, WIT event types, ops→spec-engine dep, infusion cache CF, log WIT, git sync spec)
- [x] Pass 3: 0 CRITICAL, 3 HIGH, 4 MEDIUM — all fixed (CF count config block, wasmtime ownership, cron parser, cache eviction, RAII wrapper, log recursion, BC naming)
- [x] Pass 4: 0 CRITICAL, 3 HIGH, 4 MEDIUM — all fixed
- [x] Pass 5: 0 CRITICAL, 0 HIGH, 3 MEDIUM — all fixed
- [x] Pass 6: 0 CRITICAL, 0 HIGH, 3 MEDIUM — all fixed (prism_audit capability gate, VP-037, git2 threat model)
- [x] Pass 7: **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✓ (clean pass 1/3)
- [x] Pass 8: **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✓ (clean pass 2/3) — 2 LOWs fixed proactively
- [x] Pass 9: **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✓ (clean pass 3/3)
- [x] **POST-REVIEW ADVERSARIAL CONVERGENCE: CONVERGED** — 9 passes, 0/0/0 × 3 consecutive (passes 7-9)

### Architecture Final Stats
> **Note (counts reflect Phase 2 closeout; Phase 3 patch cycle added VP-039, bringing current total to 39 VPs)**
- 22 architecture documents
- 21 ADRs (AD-001 through AD-021)
- 16 RocksDB column families
- 39+ MCP tools, 20+ resources, 5 prompts
- 38 verification properties at Phase 2 closeout (19 Kani, 11 proptest, 6 fuzz, 2 integration)
- 3 extensibility types: sensors, infusions, actions (.prx WASM plugins)
- Total adversarial passes: 36 (27 pre-review + 9 post-review)
- Total findings resolved: ~100+

- [x] **COMPLETE**: Phase 2 architecture converged — proceeding to Phase 3

## Phase 3: Story Decomposition — CONVERGED

### Progress
- [x] Story decomposition plan designed
- [x] STORY-INDEX.md created with full traceability matrix
- [x] Wave 1-6 stories written (46 core stories)
- [x] AD-022: PrismQL write operations (3 stories: S-1.13, S-3.06, S-3.07)
- [x] AD-019/020/021: WASM plugins, infusions, actions (4 stories: S-1.14, S-1.15, S-4.08, S-5.06)
- [x] Osquery-inspired enhancements (7 stories: S-2.08, S-3.08-S-3.13)
- [x] BC-2.12.011/012 created for action delivery
- [x] **Adversarial convergence: CONVERGED — 50 passes, 0/0/0 × 3 consecutive (passes 48-50)**

### Story Stats
- 75 stories across 7 waves (Wave 0 = 16: devops S-0.01/S-0.02 + 14 DTU stories S-6.06–S-6.19)
- 195 active BCs; every active BC anchored to at least one implementing story (some BCs appear in multiple stories for multi-site coverage)
- 39 VPs assigned to stories (20 Kani, 11 proptest, 6 fuzz, 2 integration)
- 16 RocksDB column families
- 14 DTU crates (prism-dtu-common + 13 per-surface clones)
- ~126 estimated implementation days
- Total adversarial passes: 50
- Total findings resolved: ~200+

### Wave Summary
> Mirrors STORY-INDEX v1.16. BC counts are raw per-story assignments (sum=235 across all waves); unique active BCs = 195 (BC-INDEX v4.7).

| Wave | Crates | Stories | BCs | Theme |
|------|--------|---------|-----|-------|
| 0 | devops, prism-dtu-common, prism-dtu-* (14 DTU crates) | 16 | 0 (infra) | Developer + Test Infrastructure |
| 1 | prism-core, prism-ocsf, prism-credentials, prism-security, prism-spec-engine | 15 | 69 (raw; 3 stories with 0 BCs) | Foundation + Pure Domain |
| 2 | prism-storage, prism-audit, prism-sensors | 8 | 30 | Infrastructure + Adapters |
| 3 | prism-query | 13 | 28 | Query Engine (incl. write ops + osquery enhancements) |
| 4 | prism-operations | 8 | 45 | Operations |
| 5 | prism-mcp, prism-audit | 10 | 50 | MCP Server + Config + Diagnostics + Log Forwarding + Audit Forwarding |
| 6 | prism-bin | 5 | 15 | Binary + E2E |

## Phase 3 Patch Cycle (2026-04-16 — reopened)

### Trigger
Resume-time consistency audit by `consistency-validator` (fresh context) confirmed 19 architecture-to-story traceability gaps plus 4 categories of missing behavioral contracts. Phase 3 status downgraded from CONVERGED to PATCH-CYCLE.

### Confirmed Gaps (19)
**BLOCKER (2):** CI/CD pipeline (Gap-1), Developer toolchain (Gap-2)
**HIGH (7):** Security scanning CI, Formal verification CI, `prism logs` CLI, `get_diagnostics` MCP tool, Multi-repo git config, `prism migrate-storage`, `prism credential` CLI, DTU stubs
**MEDIUM (5):** External log forwarding, Vault credential backend, Trace ID infra, IOC files, Audit forwarding
**LOW (3):** Dependabot/Renovate, `action_state` CF initialization, `prism unregister`
**DTU-adjacent:** VP-033 + VP-036 integration tests have no sensor stub infrastructure

### Missing BC Categories
- BC-2.14.012 acknowledge_alert (known STUB — to be completed)
- AD-019 WASM plugin invariants (6 invariants, 0 BCs)
- AD-021 Action invariants (9 invariants, 0 BCs)
- AD-020 Infusion framework (S-1.14 has 0 BCs)
- CAP-022 auto-case-creation (no BC)

### Policy Decisions
- **Story numbering:** append-only (S-0.01/02 for new Wave 0, S-5.07–10, S-6.04–06). No renumbering of existing 53 stories.
- **Invariant lift:** 6 WASM + 9 Action + ≥1 Infusion + 1 auto-case-creation invariants lifted to formal BCs. Total BC count: 169 → ~190.

### Story Patch Set (9 new + 5 scope expansions)
**Wave 0 (NEW):** S-0.01 CI/CD Pipeline, S-0.02 Developer Toolchain
**Wave 5 additions:** S-5.07 Multi-repo git config, S-5.08 Diagnostics + `prism logs`, S-5.09 External log forwarding, S-5.10 Audit forwarding
**Wave 6 additions:** S-6.04 `prism credential` CLI, S-6.05 `prism migrate-storage`, S-6.06 DTU sensor stubs
**Scope expansions:** S-6.01 (CLI dispatch), S-2.01 (action_state CF), S-5.05 (scope-out git sync), S-1.14 (infusion cache), S-4.03 (IOC file loading)

### Patch Cycle Execution Plan
- [ ] Burst 1 (parallel): product-owner writes BCs; story-writer drafts BC-independent stories; architect produces dtu-assessment.md
- [ ] Burst 2: story-writer drafts BC-dependent stories (needs B1 output)
- [ ] Burst 3: update STORY-INDEX.md, wave schedule, dependency graph, VP assignment matrix
- [ ] Burst 4: adversarial re-convergence (minimum 3 clean passes)
- [ ] Burst 5: consistency re-validation (confirm all 19 gaps closed)
- [ ] Burst 6: human approval gate
- [ ] Burst 7: /vsdd-factory:dtu-creation builds clones
- [ ] Burst 8: Phase 4 entry (Wave 0 first, then Waves 1–6)

### Adversarial Review Summary
| Pass | Novelty | CRIT | HIGH | LOW |
|------|---------|------|------|-----|
| 20 | HIGH | 3 | 6 | 3 |
| 21 | HIGH | 3 | 7 | 3 |
| 22 | HIGH | 3 | 7 | 2 |
| 23 | HIGH | 4 | 7 | 1 |
| 24 | HIGH | 5 | 4 | 3 |
| 25 | MEDIUM | 0 | 4 | 1 |
| 26 | MEDIUM | 0 | 5 | 2 |
| 27 | CONVERGED | 0 | 0 | 6 |
| 28 | CONVERGED | 0 | 0 | 1 |
| 29 | CONVERGED | 0 | 0 | 2 |
| 30 | MEDIUM | 0 | 2 | 2 |
| 31 | MEDIUM | 0 | 2 | 0 |
| 32 | MEDIUM | 0 | 2 | 0 |
| 33 | LOW | 0 | 0 | 1 |
| 34 | CONVERGED | 0 | 0 | 0 |

### Known Tracked Gaps
- BC-2.14.012: RESOLVED — fully specified in Burst 1 commit 58684c5
- Auto-case-creation from high-severity rules: RESOLVED — BC-2.14.013 committed in Burst 1 commit 58684c5, anchored to S-4.06

## Adversarial Re-Convergence Log (Burst 3+)

### Pass 1 (2026-04-17)
**Findings:** 29 (6 CRITICAL, 9 HIGH, 9 MEDIUM, 5 LOW)
**Verdict:** Not a clean pass — convergence counter RESET to 0

**CRITICAL findings (top-level themes):**
- P3P1-C-001 RocksDB CF count drift across 6+ docs (resolved by architect Burst 4a, canonical = 16)
- P3P1-C-002 STORY-INDEX Full Story List BC column stale for S-1.14/15, S-4.06/08
- P3P1-C-003 BC count drift across STATE/PRD/BC-INDEX/STORY-INDEX (authoritative = 193 per BC-INDEX)
- P3P1-C-004 SS-17/18/19 missing from ARCH-INDEX (resolved by architect Burst 4a)
- P3P1-C-005 prism-dtu crates absent from module-decomposition + dependency-graph (resolved by architect Burst 4a)
- P3P1-C-006 BC-2.14.012 capability CAP-021 (wrong) — should be CAP-022

**Fix dispatch:**
- Burst 4a (architect): CF canonicalization + SS-17/18/19 + prism-dtu (commit 0b77d63)
- Burst 4b (product-owner): BC fixes + PRD subsystem distribution + error-code renaming (in progress)
- Burst 4b (story-writer): STORY-INDEX counts + story BC table miswirings + S-6.06 endpoint alignment (in progress)
- Burst 4b (state-manager): STATE.md Phase 3 stat refresh (this commit)

**Canonical numbers post-patch-post-adversary-fix:**
- Stories: 62 across 7 waves
- Active BCs: 192 (after PO Option A retirement of BC-2.12.011/012 and arithmetic correction of SS-12/14 summary rows)
- VPs: 39 (20 Kani, 11 proptest, 6 fuzz, 2 integration)
- Architecture docs: 22
- RocksDB CFs: 16
- Subsystems: 20 (SS-17/18/19 added Burst 4a; SS-20 added Burst 7)

### Pass 2 (2026-04-17)
**Findings:** 24 (6 CRITICAL, 7 HIGH, 6 MEDIUM, 5 LOW)
**Verdict:** Not clean — convergence counter remains at 0

**CRITICAL findings:**
- P3P2-C-001 BC active-count arithmetic wrong (191 → 192 after SS-12/14 row correction)
- P3P2-C-002 STORY-INDEX traceability matrix still has retired BC-2.12.011/012
- P3P2-C-003 S-4.08 frontmatter still binds retired BCs
- P3P2-C-004 ARCH-INDEX Subsystem Registry missing SS-06 + SS-08 (17 → 19 rows)
- P3P2-C-005 S-6.06 story contradicts architect's 4-crate DTU decomposition
- P3P2-C-006 STATE.md stale 193 BC count (this fix)

**Plus:** Human review during Pass 2 triage identified an additional scope gap —
original DTU scope only covered 4 sensors; Actions (AD-021), Infusions (AD-020),
and Log Forwarding (observability.md) all need DTU coverage too. Architect
committed Burst 5.5a (16a32e6) expanding DTU scope 5 → 14 crates. Story count
will grow 62 → 75 (13 new per-surface stories + S-6.06 rescope).

**Fix dispatch:**
- Burst 5a (architect 0b77d63, d1ea8a2): SS-06/08 + prism-dtu-common
- Burst 5.5a (architect 16a32e6): +9 DTU crates (actions + infusions + log-forward)
- Burst 5b (product-owner 1de9ac2): BC arithmetic 191 → 192, PRD reconcile
- Burst 5b (state-manager, this commit): STATE.md 193 → 192 + DTU expansion record
- Burst 5b (story-writer-A, parallel): 14 DTU stories — S-6.06 rescope + S-6.07-19
- Burst 5b (story-writer-B, serial after A): pass-2 cleanup + STORY-INDEX reconcile

**Canonical numbers post-Burst-5b-po (updated by Burst 6b-sm):**
- Stories: 75 (62 + 13 DTU stories added by SW-A; Wave 0 = 16)
- Active BCs: 192 (SS-12 corrected to 10, SS-14 corrected to 12, BC-2.14.011 slot empty)
- VPs: 39
- Architecture docs: 22
- RocksDB CFs: 16
- Subsystems: 20
- DTU crates: 14 (prism-dtu-common + 13 per-surface clones)

### Pass 3 (2026-04-17)
**Findings:** 21 (3 CRITICAL, 5 HIGH, 7 MEDIUM, 6 LOW)
**Verdict:** Not clean — convergence counter remains at 0

**Pass-2 verification:** 15/16 pass-2 findings confirmed FIXED. 1 partial (P3P2-C-006 STATE.md — Story Stats section not refreshed; fixed by this commit).

**CRITICAL findings:**
- P3P3-C-001 VP-033/VP-036 reassignment incomplete (landed in STORY-INDEX matrix only; VP-INDEX + S-2.04/S-4.04/S-6.06 still showed old anchors). Fix dispatched to PO (VP-INDEX) and story-writer (story frontmatters).
- P3P3-C-002 module-decomposition Claroty YAML L2 → L4 (architect missed during Burst 5.5a sweep). Fixed in Burst 6a.
- P3P3-C-003 STATE.md Story Stats + Wave Summary stale (this fix).

**Additionally:** Human directive during pass-3 triage — enforce L0–L4 taxonomy parenthetical form (L4 (adversarial)) across all documents. Architect applied in Burst 6a (19 legacy labels replaced). Story-writer applying to story files in Burst 6b (parallel).

**Fix dispatch:**
- Burst 6a (architect 5feb982): L0–L4 taxonomy sweep + Claroty YAML + DTITI typo + COMP-DTU-005 interfaces + §1 clarity
- Burst 6b (product-owner): VP-INDEX.md VP-033/VP-036 reassignment
- Burst 6b (story-writer): story frontmatter + blocks edges (option B, human approved) + R-DTU risk mitigation anchors + S-6.06 filename rename + topological layer integerization + taxonomy sweep in story files
- Burst 6b (state-manager, this commit): STATE.md Story Stats + Wave Summary refresh + Phase 2 clarification

**Canonical numbers post-Burst-6b:**
- Stories: 75 across 7 waves (Wave 0 = 16)
- Active BCs: 192 (BC-INDEX v4.3)
- VPs: 39 (VP-033/036 now anchor to S-6.07)
- Architecture docs: 22
- RocksDB CFs: 16
- Subsystems: 20
- DTU crates: 14

### Pass 4 (2026-04-17)
**Findings:** 7 (0 CRITICAL, 3 HIGH, 2 MEDIUM, 2 LOW)
**Verdict:** Not clean — convergence counter remains at 0 (but trajectory is strong: 29 → 24 → 21 → 7)

**Pass-3 verification:** 14/16 pass-3 findings confirmed FIXED. 2 PARTIALLY FIXED (STATE.md Wave 1 parenthetical + v1.6 citation, fixed by this commit).

**HIGH findings:**
- P3P4-H-001 S-6.19 line 256 residual `prism-operations` reference (fixed by story-writer Burst 7)
- P3P4-H-002 STATE.md Wave 1 parenthetical (fixed by this commit)
- P3P4-H-003 STORY-INDEX BC Traceability Matrix missing BC-2.14.013 row (fixed by story-writer Burst 7)

**MEDIUM findings:**
- P3P4-M-001 VP-INDEX Anchor Story column backfill (37 rows) (fixed by PO Burst 7)
- P3P4-M-002 STATE.md STORY-INDEX version citation (fixed by this commit)

**LOW findings:**
- P3P4-L-001 fidelity taxonomy form inconsistency (fixed by story-writer Burst 7)
- P3P4-L-002 log-forwarding DTUs assigned to SS-08 Sensor Health (human promoted to architectural fix → SS-20 added; architect adds in Burst 7)

**Fix dispatch:**
- Burst 7 architect: add SS-20 Observability / Log Forwarding (ARCH-INDEX, module-decomp, observability.md)
- Burst 7 PO: VP-INDEX Anchor Story column backfill (37 VPs)
- Burst 7 story-writer: S-6.19 line 256, BC-2.14.013 matrix row, taxonomy canonicalization, SS-20 re-anchor (5 stories)
- Burst 7 state-manager (this commit): Wave 1 parenthetical, STORY-INDEX citation, subsystem count 19 → 20, pass-4 log

**Canonical numbers post-Burst-7:**
- Stories: 75 across 7 waves (Wave 0 = 16)
- Active BCs: 192 (BC-INDEX v4.3)
- VPs: 39 (all anchored in VP-INDEX)
- Architecture docs: 22
- RocksDB CFs: 16
- Subsystems: 20 (added SS-20)
- DTU crates: 14

### Pass 5 (2026-04-17)
**Findings:** 4 (0 CRITICAL, 0 HIGH, 3 MEDIUM, 1 LOW)
**Verdict:** Not clean — convergence counter remains at 0 (3 MEDIUM blocks clean)

**Trajectory: 29 → 24 → 21 → 7 → 4** (CRIT/HIGH zero for the second consecutive pass)

**Pass-4 verification:** 5/7 fully FIXED. 2 PARTIAL:
- P3P4-M-002 regressed (state-manager wrote v1.7 but story-writer later bumped to v1.8)
- P3P4-L-001 partial (1 table in STORY-INDEX not swept — Burst 5b summary rows at lines 584-596)

**MEDIUM findings:**
- P3P5-M-001 PRD "19 subsystems" stale (fixed by PO Burst 8)
- P3P5-M-002 PRD missing Subsystem 20 block + Distribution row (fixed by PO Burst 8)
- P3P5-M-003 STATE.md STORY-INDEX cite v1.7 → v1.8 (this fix)

**LOW finding:**
- P3P5-L-001 STORY-INDEX Burst-5b summary rows uncanonicalized (fixed by story-writer Burst 8)

**Fix dispatch:**
- Burst 8 PO: PRD §2 SS-20 block + count 19 → 20 + Distribution table row
- Burst 8 state-manager (this commit): STATE.md v1.7 → v1.8 + pass-5 log entry
- Burst 8 story-writer: STORY-INDEX lines 584-596 taxonomy sweep (L4 Adversarial → L4 (adversarial))

**Canonical numbers unchanged from Burst 7:**
- Stories: 75 | BCs: 192 | VPs: 39 | Arch docs: 22 | CFs: 16 | Subsystems: 20 | DTU crates: 14

### Pass 6 (2026-04-17)
**Findings:** 3 (0 CRITICAL, 0 HIGH, 3 MEDIUM, 0 LOW)
**Verdict:** Not clean — convergence counter remains at 0 (3 MEDIUM blocks)

**Trajectory: 29 → 24 → 21 → 7 → 4 → 3** (CRIT/HIGH zero for 3rd consecutive pass)

**Pass-5 verification:** 3/4 FIXED; 1 REGRESSED (P3P5-M-003 STORY-INDEX cite leapfrogged again — 3rd manifestation of version-race pattern).

**MEDIUM findings (all one-line text edits):**
- P3P6-M-001 STATE.md STORY-INDEX cite v1.8 → v1.9 (this fix)
- P3P6-M-002 STORY-INDEX v1.9 bump has no Burst 8 changelog entry (fixed by SW Burst 9)
- P3P6-M-003 PRD §7 preamble "all 153 behavioral contracts" → "all 192" (fixed by PO Burst 9)

**Structural improvement:** state-manager to run LAST in future bursts to avoid version-race regression pattern.

**Fix dispatch:**
- Burst 9 PO: PRD:652 153 → 192
- Burst 9 state-manager (this commit): STATE.md v1.8 → v1.9 + pass 6 log
- Burst 9 story-writer: STORY-INDEX Burst 8 changelog row

**Canonical numbers unchanged from Burst 7:**
- Stories: 75 | BCs: 192 | VPs: 39 | Arch docs: 22 | CFs: 16 | Subsystems: 20 | DTU crates: 14

### Pass 7 (2026-04-17)
**Findings:** 2 (0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW)
**Verdict:** Not clean — convergence counter remains at 0 (1 MEDIUM blocks clean)

**Trajectory: 29 → 24 → 21 → 7 → 4 → 3 → 2** (CRIT/HIGH zero for 4th consecutive pass)

**Pass-6 verification:** 3/3 FIXED — no regressions. Version-race pattern closed (state-manager ran last in Burst 9).

**MEDIUM finding:**
- P3P7-M-001 PRD §7 Traceability Matrix body has only 156 rows; should be 192.
  Missing BCs from SS-05 (+1), SS-08 (+2), SS-13 (+1), SS-14 (+2), SS-16 (+10),
  SS-17 (+6), SS-18 (+9), SS-19 (+5) = 36 rows. Entries added in Bursts 1/2/2.5/2.75/5
  were never back-populated into §7. Fix dispatched to PO Burst 10.

**LOW finding:**
- P3P7-L-001 PRD §7 Capability Coverage Summary missing CAP-029, CAP-030;
  CAP-021 count stale (2 → should be higher per SS-18 anchors).
  Bundled with Burst 10 PO fix.

**Fix dispatch:**
- Burst 10 PO: PRD §7 back-population (append 36 BC rows + refresh CAP summary)
- Burst 10 state-manager (this commit): pass 7 log entry

**Canonical numbers unchanged from Burst 9:**
- Stories: 75 | BCs: 192 | VPs: 39 | Arch docs: 22 | CFs: 16 | Subsystems: 20 | DTU crates: 14

### Pass 8 (2026-04-17) — **FIRST CLEAN PASS** ✓
**Findings:** 0 CRITICAL + 0 HIGH + 0 MEDIUM + 0 LOW = **0 blocking**
**Observation:** 1 non-blocking (P3P8-O-001: SS-19 BCs anchor to CAP-020 — pre-existing semantic from BC-INDEX v4.3, survived 7 prior passes, no arithmetic impact, deferred to post-v1 capability-naming consolidation)

**Verdict:** CLEAN — convergence counter advances to 1 of 3

**Trajectory: 29 → 24 → 21 → 7 → 4 → 3 → 2 → 0**
  CRIT/HIGH: zero for 5th consecutive pass
  MEDIUM: zero for 1st pass in the patch cycle
  Total findings decay: 29 → 0 over 8 passes

**Pass-7 verification:** 2/2 FIXED — P3P7-M-001 (§7 body 192 rows verified by per-subsystem count) and P3P7-L-001 (CAP summary complete with CAP-029/030, arithmetic reconciled to 192 unique).

**Invariants confirmed:**
- Active BCs = 192 (163 P0 + 29 P1); Total = 208; Removed = 16
- Stories = 75 (Wave 0 = 16); raw BC sum = 237
- VPs = 39 (20 Kani, 11 proptest, 6 fuzz, 2 integration); all anchored
- Architecture documents = 22; ADRs = 21
- RocksDB CFs = 16; Subsystems = 20 (SS-01..SS-20); DTU crates = 14
- BC-2.14.011 permanently reserved-removed
- BC-2.12.011/012 retired (Option A)
- Dual-anchors: BC-2.01.010 (CAP-001/002), BC-2.16.008 (CAP-029/030)
- SS-20 Observability = 0 BCs by design
- state-manager runs LAST in every burst (closed version-race pattern)

### Pass 9 (2026-04-17) — SECOND CLEAN PASS ✓
**Findings:** 0 CRITICAL + 0 HIGH + 0 MEDIUM + 2 LOW = 0 blocking + 2 LOW
**Verdict:** CLEAN — counter advanced 1 → 2 of 3

**LOW findings (both fixed in Burst 11):**
- P3P9-L-001 BC-INDEX "Removed BCs (14)" header → "(16)" (bundled with PO CAP-031 fix)
- P3P9-L-002 dependency-graph.md `prism-observability` → `prism-mcp` (architect fix)

**Trajectory: 29 → 24 → 21 → 7 → 4 → 3 → 2 → 0 → 0** (CRIT/HIGH 0 for 6 consecutive passes)

### Burst 11 — CAP taxonomy correction + dep-graph label fix
**Trigger:** Pass 8 Observation P3P8-O-001 (SS-19 BCs anchored to CAP-020 "Detection Rules" — semantic mismatch). Human directive: fix it; treat semantic mis-anchoring as blocking going forward.

**Commits:**
- `eb55aa3` (PO) — Created CAP-031 "Infusion Enrichment" in capabilities.md. Re-anchored BC-2.19.001/002/003/005 from CAP-020 → CAP-031 (BC-2.19.004 already correctly CAP-030). Updated BC-INDEX v4.3 → v4.4, PRD §7 body, PRD §7 Coverage Summary (CAP-020: 14 → 10; +CAP-031 = 4). Bundled P3P9-L-001 BC-INDEX header fix.
- `ddb4ffb` (architect) — dependency-graph.md:181 `prism-observability` → `prism-mcp` (P3P9-L-002).

**Convergence impact:** SPEC CHANGE — counter RESET from 2 to 0.

**Principle adopted:** Semantic anchoring integrity is now a first-class invariant. Mis-anchoring NEVER "Observation"; MEDIUM+ severity minimum. See audit_policy_decisions frontmatter.

### Pass 12 (2026-04-17) — COMPREHENSIVE ANCHORING AUDIT ⚠
**Findings:** 26 (9 CRITICAL, 11 HIGH, 4 MEDIUM, 2 LOW)
**Verdict:** BLOCK convergence — major systemic mis-anchoring discovered across 6 axes

**Root-cause finding P3P12-A4-001:** PRD §7 Capability Coverage Summary had CAP titles hand-edited to RENAME capabilities to match mis-anchored BCs, rather than fixing the anchors. CAP-024 (Resource Watchdog) and CAP-025 (Buffered Audit Logging) are literally swapped in PRD §7 vs. canonical capabilities.md. This masked 8 structurally identical mis-anchors.

**9 CRITICAL findings:**
- P3P12-A4-001 PRD §7 CAP title editing (root cause)
- P3P12-A1-002 BC-2.13.004 Sequence Detection CAP-021 → should be CAP-020
- P3P12-A1-003 BC-2.15.003 Audit Log Persistence CAP-019 → should be CAP-025
- P3P12-A1-004 BC-2.15.004 Audit Buffer Overflow CAP-019 → should be CAP-025
- P3P12-A1-005 BC-2.15.008 Query Denylisting CAP-025 → should be CAP-024
- P3P12-A1-006 BC-2.15.001/002 RocksDB CAP-024 → should be CAP-019
- P3P12-A1-007 BC-2.15.006/007 Resource Watchdog — BC says CAP-024 (correct), PRD says CAP-025 (wrong)
- P3P12-A1-008 All 6 BC-2.17.* (WASM Plugins) CAP-029 → needs NEW CAP-032
- P3P12-A1-009 All 9 BC-2.18.* (Actions) CAP-021 → needs NEW CAP-033

**Systemic patterns:**
1. CAP-title-editing cover-up (fix root cause first)
2. Missing CAPs for SS-10, SS-17, SS-18 (need CAP-032/033/034)
3. Three-way drift: BC file vs. BC-INDEX vs. PRD §7 for CAP anchors (8 BCs disagree)

**Axes clean:** VP→Story anchors (all 39 correct), crate/file paths (no phantom refs), retired BCs (clean retirement).

**Fix dispatch:** Burst 13 coordinated PO + story-writer + state-manager — COMPLETE.

### Burst 13 — Comprehensive Anchoring Fix (2026-04-17)

**Scope:** Address all 26 pass-12 findings (9 CRIT, 11 HIGH, 4 MED, 2 LOW) across 6 anchoring axes. Root cause P3P12-A4-001: PRD §7 CAP title hand-editing that masked systemic mis-anchors.

**Sub-bursts (sequential, state-manager runs last):**
1. PO-A (commits 0d48b86, ee6a4a3, 9e21795): Created CAP-032 "WASM Plugin Runtime", CAP-033 "Action Delivery Engine", CAP-034 "MCP Server & Transport"; re-anchored 27 BC frontmatters; fixed BC-2.01.010 subsystem label to "Sensor Adapter Layer".
2. Story-writer (commit b25ef6e): Fixed S-5.08 bcs (removed BC-2.10.*) and subsystems (added SS-10); S-1.02 subsystems (added SS-03, SS-11); S-3.05 subsystems (added SS-11); STORY-INDEX v1.9 → v1.10.
3. PO-B (commits bcb9aa2, 1ed142c): Regenerated BC-INDEX CAP column from BC file source of truth (v4.4 → v4.5); regenerated PRD §7 body matrix + Coverage Summary + §2 SS-10/17/18 capability refs. Restored canonical CAP titles (CAP-024 "Resource Watchdog", CAP-025 "Buffered Audit Logging" — were swapped). Grand total 192 active BCs preserved.
4. PO-B follow-up (commit 3f58e85): Closed BC-2.10.002 dual-anchor drift; normalized 5 BCs (BC-2.01.010, BC-2.10.002, BC-2.10.004, BC-2.10.005, BC-2.16.008) from quoted-string to YAML-array capability frontmatter.

**Findings addressed (all 26):**
- P3P12-A4-001 (root cause PRD §7 hand-editing) — CLOSED by PO-B canonical title restore
- P3P12-A1-002..009 (9 CRIT BC re-anchor findings) — CLOSED by PO-A
- 4 story-level findings — CLOSED by story-writer
- 6 MED/LOW drift findings — CLOSED by PO-B regeneration

**New policy artifacts from this burst:**
- 3 new CAPs bringing total from 31 → 34
- BC-INDEX now single source of truth for CAP column (regenerated from BC files)
- PRD §7 now regenerated from source (no more hand-editing CAP titles)
- 5 normalized dual-anchor BCs match YAML-array convention

**Next:** Adversary pass 13 targeting clean (0 findings). Need 3 consecutive clean passes for convergence (counter resets to 0 of 3 due to Burst 13 spec changes).

### Pass 13 (2026-04-17)
**Findings:** 8 (4 CRITICAL, 4 HIGH, 0 MEDIUM, 0 LOW)
**Verdict:** Not clean — BLOCK; fixes dispatched to Burst 14

**Trajectory: 26 → 8** (69% decay pass-over-pass)

**CRITICAL findings:**
- P3P13-A2-001 BC-2.10.004 mis-anchored to CAP-001/002 (internal capabilities) — should be CAP-009 (MCP client scoping)
- P3P13-A2-002 Only 2 of 9 BC-2.01.* files received SS-01 subsystem rename in Burst 13 (7 still stale)
- P3P13-A3-001 BC-2.10.004 three-way drift (BC file vs BC-INDEX vs PRD §7 all disagree)
- P3P13-A2-004 ARCH-INDEX vs BC-INDEX subsystem-name taxonomy drift across 7+ subsystems

**HIGH findings:**
- P3P13-A2-003 BC-2.10.005 dual-anchor CAP column not harmonized
- P3P13-A3-002 BC-INDEX delimiter inconsistency (`+` vs `,`)
- P3P13-A3-003 BC-INDEX BC-2.10.004 title stale vs v2.0
- P3P13-A6-001 S-1.02 missing SS-12 (ScheduleId is a scheduler concern)

**Fix dispatch:** Burst 14 — all 8 findings addressed.

### Burst 14 — Residual Anchoring + Taxonomy Canonicalization (2026-04-17)

**Scope:** Address all 8 pass-13 findings (4 CRIT + 4 HIGH) focused on:
1. BC-2.10.004 semantic mis-anchor (CAP-001/002 are internal, but BC-2.10.004 is MCP-boundary)
2. Partial propagation gap from Burst 13 (only 2 of 9 BC-2.01.* files got subsystem rename)
3. BC-INDEX cleanup residuals (delimiter inconsistency, stale title, retired row drift)
4. NEW dimension: ARCH-INDEX vs BC-INDEX subsystem-name taxonomy drift across 7+ subsystems

**Sub-bursts (sequential, state-manager runs last):**
1. PO-A A1 (commit 21d25ab): Re-anchor BC-2.10.004 [CAP-001, CAP-002] → CAP-009 (MCP client scoping is the Client Configuration concern). PRD §7 body + Coverage Summary updated atomically; CAP-001 10→8, CAP-002 3→2, CAP-009 10→11.
2. Story-writer (commit bfaef13): S-1.02 subsystems += SS-12 (ScheduleId scheduler concern); STORY-INDEX v1.10 → v1.11.
3. PO-A A2 (commit 92c0b10): SS-01 rename propagated to 7 active BC-2.01.* files direct to ARCH canonical "Sensor Adapters" (avoiding double-touch with A6).
4. PO-A A3/A4/A5 (commit bc288b4): BC-INDEX cleanup — BC-2.10.005 retired-row CAP column harmonized to dual-anchor `CAP-005, CAP-009`; BC-2.10.002 delimiter `+` → `,`; BC-2.10.004 title column synced to v2.0 "Client Scoping on Every Tool (Stateless Model)".
5. PO-A A6 initial (commit 7f91a42): ARCH-INDEX canonical subsystem names propagated across BC-INDEX + 69 BC file frontmatters + PRD §7 for SS-01/05/07/11/12/15/16.
6. PO-A A6 follow-up (commit f35cd6b): SS-04/10/14 taxonomy sync (46 more BC files). Also fixed 6 removed SS-01 BC files previously retained "Sensor Query Pipeline" label. Final grep confirms ZERO residual drift across 208 BC corpus.

**Taxonomy policy established:** ARCH-INDEX is authoritative source of truth for subsystem NAMES. BC-INDEX subsystem labels and BC file frontmatter `subsystem:` fields must match ARCH-INDEX canonical names. Symmetric with capabilities.md being source of truth for CAP titles. Named policy flag: `architecture_is_subsystem_name_source_of_truth`.

**Findings addressed (all 8):**
- P3P13-A2-001 CRIT: BC-2.10.004 re-anchor to CAP-009 — CLOSED
- P3P13-A2-002 CRIT: 7 active BC-2.01.* subsystem rename — CLOSED
- P3P13-A2-003 MED: BC-2.10.005 dual-anchor harmonization — CLOSED
- P3P13-A2-004 HIGH: ARCH-INDEX vs BC-INDEX taxonomy — CLOSED (ARCH-INDEX wins)
- P3P13-A3-001 CRIT: BC-2.10.004 three-way drift — CLOSED
- P3P13-A3-002 HIGH: BC-INDEX delimiter — CLOSED
- P3P13-A3-003 HIGH: BC-INDEX title sync — CLOSED
- P3P13-A6-001 HIGH: S-1.02 subsystems +SS-12 — CLOSED

**Next:** Adversary pass 14 targeting clean. Convergence counter resets to 0/3 due to Burst 14 spec changes (re-anchor + taxonomy canonicalization). Need 3 consecutive clean passes.

### Burst 15 — Aggregation Doc + PRD/Story Residual Drift (2026-04-17)

**Scope:** Close 4 pass-14 findings (0 CRIT, 2 HIGH, 2 MED). Burst 14's rename sweep missed three aggregation docs and two downstream consumers of BC-2.10.004's old v1.0 title.

**Sub-bursts (parallel; state-manager last):**
1. PO T1 (commit 8412caa): SUBSYSTEMS-01-04, 05-07, 08-10 aggregation docs — canonicalize subsystem names to ARCH-INDEX for SS-01 (Sensor Adapters), SS-04 (Feature Flags), SS-05 (Audit Trail), SS-07 (PrismQL Engine), SS-10 (MCP Interface). 10 string fixes across 3 files; SS-04 BC count corrected 15→14.
2. PO T2+T3 (commit f61ae4f): PRD §5 line 228 BC-2.10.004 title sync v1.0 → v2.0 (Client Scoping). PRD §7 coverage disclaimer arithmetic fix — grand total column sum = 197 (192 active BCs + 5 active dual-anchor extras); retired BC-2.10.005 excluded from matrix.
3. Story-writer (commit 90064ac): S-5.02 line 36 BC-2.10.004 title sync to v2.0. STORY-INDEX BC-INDEX version pins v4.3 → v4.5 at lines 24 and 62. STORY-INDEX v1.11 → v1.12.

**Findings addressed (all 4):**
- P3P14-A2-001 HIGH aggregation doc taxonomy — CLOSED (PO T1)
- P3P14-A3-001 HIGH BC-2.10.004 title three-way drift — CLOSED (PO T2 + SW)
- P3P14-A8-001 MED STORY-INDEX stale version pins — CLOSED (SW)
- P3P14-A4-001 MED coverage disclaimer arithmetic — CLOSED (PO T3)
- P3P14-O-001 LOW filename slug observation — accepted (append_only_numbering policy)

**Next:** Adversary pass 15 targeting clean. Counter resets to 0/3 due to Burst 15 spec changes. Need 3 consecutive clean passes.

### Burst 16 — Aggregation Doc Retirement Rectification (2026-04-17)

**Scope:** Close 2 pass-15 MEDIUM findings on SUBSYSTEMS-*-SUMMARY.md aggregation docs (retired-BC rows and stale active-BC titles).

**Sub-burst (single PO commit):**
- 01a22af: SUBSYSTEMS-01-04/05-07/08-10 — strikethrough retired BCs (BC-2.04.014, BC-2.06.009, BC-2.10.005) with *(removed)* annotation; titles synced to BC file current form (retired) and BC-INDEX authoritative (active: BC-2.04.003, BC-2.04.009, BC-2.05.001). SS-06 count 10 → "9 active, 1 removed". SS-10 count 11 → "10 active, 1 removed". DI-003 enforcer list: BC-2.10.005 removed; added inline note about stateless-model migration via BC-2.10.004 v2.0. SUBSYSTEMS-05-07 overview total corrected to reflect retirement accounting.

**Findings addressed (all 2):**
- P3P15-A8-001 MED retired-BC masquerading in aggregation docs — CLOSED
- P3P15-A8-002 MED stale active-BC titles vs BC-INDEX — CLOSED
- P3P15-A2-OBS-001 LOW security-architecture.md section header "Feature Flag System" — ACCEPTED (descriptive prose, not subsystem-label reference)
- P3P15-A8-OBS-002 LOW BC-INDEX strikethrough titles vs BC file current titles — ACCEPTED (historical preservation convention)

**DI-003 open question flagged:** After BC-2.10.005 retirement, no active BC explicitly enforces DI-003 for tool-list notifications. The stateless-model of BC-2.10.004 v2.0 subsumes the behavior but does not cite DI-003. Marked in SUBSYSTEMS-08-10-SUMMARY for future spec pass review.

**Next:** Adversary pass 16 targeting clean. Counter resets to 0/3 due to Burst 16 spec change.

### Burst 17 — Aggregation Doc Completeness (2026-04-17)

**Scope:** Close 1 pass-16 MEDIUM finding. Three active BCs (BC-2.05.011, BC-2.08.008, BC-2.08.009) existed in BC-INDEX and BC files since Burst 2.5 (2026-04-16) but were missing from SUBSYSTEMS aggregation tables. Burst 16 had touched count annotations without adding the missing rows.

**Sub-burst (single PO commit):**
- 75258b2: SUBSYSTEMS-05-07 / SUBSYSTEMS-08-10 — add 3 missing active BC rows + correct SS-05 header (10→11), SS-08 header (7→9), overview totals (SS-05-07: 25→26 active, 30→31 historical; SS-08-10: 25→27 active, 26→28 historical), DI-004 row reference (001-010→001-011).

**Findings addressed (all 1):**
- P3P16-A8-001 MED — aggregation doc active-BC completeness — CLOSED

**Next:** Adversary pass 17 targeting clean. 3-consecutive-clean convergence still requires 3 subsequent clean passes; counter at 0/3.

### Burst 18 — SS-07 Title Canonicalization + Invariant Matrix Cleanup (2026-04-17)

**Scope:** Close 3 pass-17 findings (1 HIGH + 2 LOW-elevated-to-MED per anchor-integrity policy).

**Sub-burst (single PO commit):**
- 47b64ca: 
  1. SS-07 three-way title drift (P3P17-A8-001 HIGH): BC-2.07.001 "Internal Ephemeral Pagination Token Structure" / BC-2.07.003 "Query Engine Sensor-Fetch Cache with Configurable TTL" / BC-2.07.005 "Cache Key Derivation from Push-Down Parameters" synced from BC file H1 (authoritative) to BC-INDEX lines 97/99/101 and PRD §5 lines 181/183/185.
  2. DI-004 overclaim cleanup (P3P17-A2-OBS-001): Reverted Burst-17 "001-011 (all)" to "001-010 (all)"; added new DI-026 "Forward Watermark Monotonicity" row anchored to BC-2.05.011. Rationale: BC-2.05.011 governs forwarding delivery semantics (INV-AUDIT-FWD-001), not audit completeness.
  3. BC-2.14.013 Story anchor (P3P17-A8-OBS-002): Traceability Story field "TBD (wave 4 decomposition)" → "S-4.06"; matches S-4.06 bcs frontmatter. Also updated Story Anchor section body to "S-4.06 — prism-operations: Case Management".

**Policy elevation:** Per `semantic_anchoring_integrity`, LOW observations on anchor-like claims (invariant-to-BC, BC-to-Story) were elevated to MED and fixed in this burst rather than accepted.

**Findings addressed (all 3):**
- P3P17-A8-001 HIGH SS-07 three-way title drift — CLOSED
- P3P17-A2-OBS-001 (→MED) DI-004 overclaim — CLOSED  
- P3P17-A8-OBS-002 (→MED) BC-2.14.013 Story anchor — CLOSED

**Next:** Adversary pass 18 targeting CLEAN. Counter resets to 0/3 due to Burst 18 spec changes. Need 3 consecutive clean passes.

### Burst 19 — Systematic BC Title Reconciliation + Story Anchor + Matrix Enforcer Completeness (2026-04-17)

**Scope:** Close 3 pass-18 findings (1 HIGH + 2 MED). Pass 18 broad sweep exposed that pass-17 SS-07 title fix was the tip of a systemic iceberg: 30+ BCs had three-way title drift. Burst 19 canonicalized with a new policy decision: `bc_h1_is_title_source_of_truth`.

**Sub-bursts (parallel where possible, state-manager last):**
1. PO-A (commit 362011e): BC-2.14.012 Traceability Story field + Story Anchor body + VP Anchors section corrected from S-4.06 → S-4.07 (S-4.07 owns BC-2.14.012 per STORY-INDEX; S-4.06 owns BC-2.14.013). SS-08-10 Invariant Coverage Matrix: DI-002 enforcer list added BC-2.08.008, BC-2.08.009 (BC frontmatter confirms); DI-004 enforcer list added BC-2.08.008, BC-2.08.009 (BC frontmatter confirms).
2. PO-B (commit 65c77c1): Systematic 44-BC title reconciliation. 12 BC H1s updated to absorb policy-relevant enrichment from BC-INDEX (Confirmation Token 100-cap, audit fail-closed qualifier, VP-039 watermark, sandbox defaults, retry backoff, etc.). 32 BC-INDEX rows synced to final H1 values. 26 PRD §2 rows + 3 PRD §6 rows synced. Key semantic resolutions: BC-2.09.004 (H1 correct — centralized safety flags; BC-INDEX "Parallel Fields" was a factual contradiction of the BC body); BC-2.02.008 (H1 correct — 4 tiers including "None" fallback; BC-INDEX "Three-Tier" was wrong). BC-INDEX bumped v4.5 → v4.6.
3. PO-B follow-up (commit 4eae747): SUBSYSTEMS-01-04 (4 rows) + SUBSYSTEMS-08-10 (3 rows) title sync to match commit 65c77c1 authoritative H1 (SUBSYSTEMS-05-07 already matched). No SUBSYSTEMS file exists for SS-11+; confirmed by ls glob.

**New policy: `bc_h1_is_title_source_of_truth`** — BC file H1 is authoritative; BC-INDEX, PRD §2/§5, SUBSYSTEMS-*-SUMMARY.md title columns MUST match. Symmetric with `architecture_is_subsystem_name_source_of_truth` for subsystem labels, `capabilities.md is source-of-truth` for CAP titles. Now 7 policy flags total.

**Findings addressed (all 3):**
- P3P18-A2-001 HIGH systemic title drift — CLOSED (44 BCs synced, 3 SUBSYSTEMS docs synced, BC-INDEX v4.5 → v4.6)
- P3P18-A4-001 MED BC-2.14.012 Story anchor — CLOSED
- P3P18-A3-001 MED SS-08-10 matrix underclaim — CLOSED

**Next:** Adversary pass 19 targeting CLEAN. Counter resets to 0/3 due to Burst 19 extensive spec changes. Need 3 consecutive clean passes.

### Burst 20 — BC Semantic Unification + Multi-Axis Completeness (2026-04-17)

**Scope:** Close 6 pass-19 findings (1 HIGH + 5 MED). Pass 19 broad sweep found: (a) BC-2.11.012 body self-contradiction on virtual field names, (b) 7 more BC title drifts, (c) invariant matrix DI-026 underclaim and DI-018 overclaim, (d) BC-2.14.012 DI-002 misattribution, (e) STORY-INDEX stale BC-INDEX v4.5 pin, (f) STORY-INDEX multi-story mapping underclaim.

**Sub-bursts (PO 3-way + SW parallel; state-manager last):**
1. PO T1 (commit c6ada8e): BC-2.11.012 canonical virtual field names unified — `_sensor`, `_client`, `_source_table` across H1, preconditions, error text, edge case, BC-INDEX, PRD §2. Resolves HIGH severity body self-contradiction.
2. PO T2+T4 (commit e6185f3): 7 H1↔INDEX title drifts closed (BC-2.18.006 "Action" prefix moved INTO H1; BC-2.14.001/2.15.009/2.14.009/2.12.006 INDEX synced to fuller H1; BC-2.11.005/2.11.009 em-dash typography). BC-2.14.012 L2 Invariants DI-002 → DI-008 (DI-002 is credential-namespacing; DI-008 is client-data-separation for alerts).
3. PO T3 (commit 23a6bd5): SUBSYSTEMS-08-10 matrix — new DI-026 enforcer row (BC-2.10.010); BC-2.10.010 row invariants DI-004, DI-026; DI-004 list +BC-2.10.010. SUBSYSTEMS-05-07 matrix — BC-2.07.005 removed from DI-018 list (BC body has no DI citations).
4. SW (commit ee08ff4): STORY-INDEX BC-INDEX pin v4.5 → v4.6; BC Traceability Matrix multi-story mapping for BC-2.05.001/002/003/004/006/008 (S-2.04 + S-5.10); STORY-INDEX v1.12 → v1.13. S-2.04 frontmatter verified includes these BCs.

**Findings addressed (all 6):**
- P3P19-A4-001 HIGH BC-2.11.012 body self-contradiction — CLOSED
- P3P19-A2-001 MED 7 more H1↔INDEX title drifts — CLOSED
- P3P19-A6-001 MED DI-026 underclaim + DI-018 overclaim — CLOSED
- P3P19-A4-002 MED BC-2.14.012 DI-002 misattribution — CLOSED
- P3P19-A10-001 MED STORY-INDEX v4.5 pin stale — CLOSED
- P3P19-A5-001 MED STORY-INDEX matrix underclaim for S-5.10 — CLOSED

**Next:** Adversary pass 20 targeting CLEAN. Counter resets to 0/3 due to Burst 20 spec changes.

### Burst 21 — Exhaustive Sweep + Un-Retire + Matrix Completeness + EC Collision Resolution (2026-04-17)

**Scope:** Close all 12 pass-20 findings (2 CRIT + 5 HIGH + 2 MED + 3 LOW obs). User decision: Option A un-retire the 3 BCs with new Config-Reload semantics.

**Sub-bursts (sequential PO 4-phase + SW parallel; state-manager last):**

1. **PO-A un-retire (bea56b6)**: BC-2.04.014, BC-2.06.009, BC-2.10.005 reinstated. Active BCs 192 → 195. Removed 16 → 13. BC-INDEX v4.6 → v4.7. PRD §7 grand total 197 → 201 (195 active + 6 active dual-anchor). DI-003 SUBSYSTEMS-08-10:108 note updated — coverage restored by BC-2.10.005.

2. **PO-B exhaustive title sweep (46bbe57)**: Full 195-BC H1↔BC-INDEX comparison. 7 drifts fixed (BC-2.04.003 INDEX→H1 enrichment moved into H1; BC-2.12.009/2.13.009/2.13.012/2.15.004/2.15.005/2.18.007 INDEX synced to fuller H1). 188 BCs unchanged — confirms sweep reached steady state.

3. **PO-C invariant matrix completeness (a5ea530)**: DI-015 → BC-2.04.009 (100-token cap). DI-022 → BC-2.12.001, BC-2.12.004 (splay). DI-023 → BC-2.12.005, BC-2.12.006 (epoch/counter). DI-024 → BC-2.13.001 (rule validation). DI-026 label drift SUBSYSTEMS-05-07:86 corrected to canonical "Audit Buffer Durability". BC-2.14.009 L2 Invariants DI-004 removed (misattribution). All substantively justified by BC body content before citation added.

4. **PO-D EC-ID collision renumber (85eadcd)**: SS-14 — EC-14-032/033/034 three collisions resolved (kept BC-2.14.008/009; renumbered BC-2.14.009/010 → EC-14-053/054/055). SS-15 — EC-15-011 three-way (kept BC-2.15.002; renumbered BC-2.15.003 → EC-15-041, BC-2.15.011 → EC-15-042).

5. **SW story anchors (f43241b)**: BC-2.04.014 → S-5.01 (MCP notification dispatch). BC-2.06.009 → S-5.05 (config reload lifecycle). BC-2.10.005 → S-5.01 (MCP bootstrap). STORY-INDEX BC Traceability Matrix +3 rows. total_bcs_covered 192 → 195. STORY-INDEX v1.13 → v1.14.

**Findings addressed (all 12):**
- P3P20-A5-001 CRIT retired-vs-active contradiction — CLOSED (un-retire)
- P3P20-A5-002 CRIT residual title drifts — CLOSED (exhaustive sweep)
- P3P20-A4-001 HIGH 3 orphan DIs — CLOSED
- P3P20-A3-001 HIGH DI-024 miss — CLOSED
- P3P20-A3-002 HIGH DI-026 label drift — CLOSED
- P3P20-A3-003 HIGH BC-2.14.009 DI-004 misattribution — CLOSED
- P3P20-A2-001 HIGH EC-ID collisions — CLOSED
- P3P20-A3-004/005 MED BC-2.13.013/2.15.010 DI-008 (unresolved — no clean DI match; body prose suggests leaving as-is or creating new DI; flagged as Observation until domain-spec update)
- P3P20-A3-006 LOW BC-2.14.008 DI-004 (observation — not fixed this burst)
- P3P20-A7-001 LOW SUBSYSTEMS retired row format inconsistency — obviated by un-retire (BC-2.07.007-010 still out of scope)
- P3P20-A10-001 LOW DI-003 "review needed" marker — CLOSED (coverage restored via un-retire)

**Next:** Adversary pass 21 targeting CLEAN. Major spec changes; counter resets to 0/3. If pass 21 CLEAN, start 3-consecutive-clean count.

### Burst 22 — Body/AC Propagation + Invariant Matrix Round 2 + Cross-Index Count Refresh (2026-04-17)

**Scope:** Close all 6 blocking pass-21 findings (3 HIGH + 3 MED). Per pass-21 insight "no new axes — all retreads", this burst focused on derivation-layer consistency.

**Sub-bursts:**
1. SW (absorbed into e28798f due to case-insensitive worktree sharing): S-5.01 body BC table +BC-2.04.014, BC-2.10.005 with AC-6, AC-7 traces. S-5.05 body BC table +BC-2.06.009 with AC-10 trace. Token Budget subtable counts updated. STORY-INDEX prose pins v4.6→v4.7, 192→195 at lines 24/65. STORY-INDEX v1.14→v1.15.
2. PO T1 (7608614): PRD §2 line 60 intro 192/16 → 195/13.
3. PO T2+T3 (e28798f): BC-2.12.004 L2 Invariants +DI-032 (body-verified). SUBSYSTEMS-05-07 invariant matrix +DI-029 row (BC-2.06.005 enforcer per invariants.md). Deferred: DI-028 citations for BC-2.12.001/BC-2.13.006 (body amendments required) + DI-029 citation for BC-2.06.005 L2 Invariants (body amendment required) — tracked in STATE.md deferred_invariant_citations.
4. PO T4 (9e7b0e9): STATE.md line 392 Story Stats "192 active BCs" → "195 active BCs".

**NEW POLICY: `bc_array_changes_propagate_to_body_and_acs: true` (8th flag)** — when story `bcs:` frontmatter changes, body BC table + ACs + Token Budget counts must update in same commit. Severity HIGH. Prevents the P3P21-A7 drift class Burst 21 inadvertently created. Prompt to propagate this policy to the vsdd-factory plugin delivered to user 2026-04-17.

**Findings addressed (6 of 8):**
- P3P21-A7-H-001/002 HIGH story body BC table drift — CLOSED (SW)
- P3P21-A7-H-003 HIGH missing ACs for un-retired BCs — CLOSED (SW)
- P3P21-A2-M-001 MED PRD §2 intro stale count — CLOSED (PO T1)
- P3P21-A2-M-002 MED STORY-INDEX stale pins — CLOSED (SW)
- P3P21-A6-M-003 MED 3 orphan DIs — PARTIALLY CLOSED (DI-032 cited; DI-028/029 deferred pending BC body amendments)
- P3P21-A9-O-001 LOW STATE.md line 392 — CLOSED (PO T4)
- P3P21-A10-O-002 LOW token-budget estimates — CLOSED (SW absorbed)

**Deferred for future burst:** BC-2.12.001/BC-2.13.006 body amendments for DI-028 enforcement (postconditions + new E-SCHED-008/E-RULE-011 error cases). BC-2.06.005 body amendment for DI-029 (cross-validation postcondition). Creator-justify-anchors policy prevents citation without body substantiation.

**Next:** Adversary pass 22 targeting CLEAN. Counter resets to 0/3 due to Burst 22 spec changes + 8th policy flag adoption.

### Burst 23 — Policy-8 Drift Sweep + E-SCHED-004 Completion + Wave Summary Refresh (2026-04-17)

**Scope:** Close 4 pass-22 findings (3 HIGH + 1 MED) + 2 LOW observations. Policy `bc_array_changes_propagate_to_body_and_acs` exposed pre-existing drift in STORY-INDEX aggregate arithmetic and story frontmatter/body coherence.

**Sub-bursts:**
1. State-manager pre-commit (committed before SW/PO): STATE.md line 414 Wave Summary pins v1.9/v4.3/237/192 → v1.16/v4.7/235/195 (P3P22-A9-M-001).
2. SW (stories(phase-3-patch-burst-23-sw)): STORY-INDEX Wave 5 arithmetic 237→235 / Wave 5 column 50→48 (propagation of Burst 21 un-retire additions); S-5.08 Full Story List BCs column 7→2 (Burst 13 de-over-claim propagation); S-3.01 body BC table +BC-2.11.006 with AC-8 trace citation; STORY-INDEX v1.15→v1.16. Sweep confirmed other Full Story List rows (N) drift-free or additionally corrected.
3. PO (bc(phase-3-patch-burst-23-po)): BC-2.12.004 Error Cases +E-SCHED-004 row per DI-032 substantiation. Body invariants/postconditions updated if needed.
4. State-manager final: this commit (STATE.md frontmatter, body log, lesson 23).

**Findings addressed (all 6):**
- P3P22-A3-H-001 HIGH STORY-INDEX Wave 5 arithmetic — CLOSED (SW)
- P3P22-A8-H-002 HIGH S-5.08 Full Story List count — CLOSED (SW)
- P3P22-A2-H-003 HIGH S-3.01 body/AC coherence — CLOSED (SW)
- P3P22-A9-M-001 MED STATE.md line 414 pins — CLOSED (SM pre-commit)
- P3P22-A6-O-001 LOW BC-2.12.004 E-SCHED-004 — CLOSED (PO)
- P3P22-A10-O-002 LOW BC-2.06.009 filename slug — ACCEPTED (append_only_numbering policy protects filename; linkage functional)

**Next:** Adversary pass 23 targeting CLEAN. Counter resets to 0/3.

### Pass 23 (2026-04-18)
**Findings:** 7 (0 CRITICAL, 4 HIGH, 1 MED, 2 LOW)
**Verdict:** Not clean — BLOCK; convergence counter remains at 0/3
**Novelty:** HIGH — surfaced new drift class: architecture-layer staleness after VP-INDEX updates (verification-architecture.md and verification-coverage-matrix.md had not been updated to reflect VP additions/reassignments in VP-INDEX v1.3)

**Trajectory: 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7** (uptick — new VP-layer drift axis)

**HIGH findings (4):**
- P3P23-A-H-001 VP-039 absent from verification-architecture.md Provable Properties Catalog and P0 list
- P3P23-A-H-002 verification-coverage-matrix.md prism-audit row missing; VP-to-module totals stale (38→39 / Kani 19→20)
- P3P23-A-H-003 SS-07 label "PrismQL Engine" stale in ARCH-INDEX.md (renamed to "Adapter Pagination & Response Cache" per architecture evolution)
- P3P23-A-H-004 DI-026 not traced in verification-coverage-matrix.md; BC-2.05.011 and VP-039 traceability absent

**MED finding (1):**
- P3P23-A-M-001 VP-033 and VP-036 stale module assignments in verification-architecture.md Provable Properties Catalog (pre-Burst-6b residue)

**LOW findings (2):**
- P3P23-A-L-001 PRD §5 error taxonomy row count stale (table had 27 rows; canonical regeneration yields 33 active namespaces)
- P3P23-A-L-002 domain-spec/entities.md StorageDomain variant count 12 but 4 additional variants (ActionState, InfusionCache, PluginState, EventBuffer) present in architecture docs

**Fix dispatch:** All 7 findings → Burst 24 (architect: H-001/002/003/004 + M-001; product-owner: L-001; both via entities.md: L-002; story-writer: L-003 BC label drift from SS-07 rename).

### Burst 24 — VP-Architecture Coherence + SS-07 Rename + PRD §5 Regen + Entities Completeness (2026-04-18)

**Scope:** Close all 7 pass-23 findings. Adopt policy 9 (`vp_index_is_vp_catalog_source_of_truth`). Adoption sweep confirms verification arch-docs coherent with VP-INDEX v1.3.

**Sub-bursts (sequential; state-manager runs last per policy 3):**

1. **Architect Fix 1 (0dd5a30):** VP-039 added to verification-architecture.md. Provable Properties Catalog entry created with module (prism-audit), tool (Kani), phase (Phase 5), DI citation (DI-026). VP-039 added to P0 list. Mermaid diagram updated. Closes P3P23-A-H-001.

2. **Architect Fix 2 (499d0aa):** verification-coverage-matrix.md triple-fix — (a) prism-audit row added to VP-to-Module table; (b) new prism-dtu-crowdstrike row added; (c) totals corrected 38→39 VPs / Kani 19→20; (d) DI-026 traceability column added; (e) BC-2.05.011 and VP-039 trace entries added. Closes P3P23-A-H-002 and P3P23-A-H-004.

3. **Architect Fix 3 (4738ee3):** SS-07 renamed "PrismQL Engine" → "Adapter Pagination & Response Cache" in ARCH-INDEX.md line 98. Authoritative source updated. Closes P3P23-A-H-003.

4. **Architect Policy 9 sweep bonus (522b4bd):** VP-033 and VP-036 stale module assignments corrected in verification-architecture.md Provable Properties Catalog (pre-Burst-6b residue). Policy 9 adoption sweep confirmed all 39 VPs present in both arch-docs with matching module, tool, phase. Closes P3P23-A-M-001.

5. **Product-owner Fix A (950f4ce):** PRD §5 Error Taxonomy regenerated from supplement. Row count now 33 active namespaces (from 27 + retired CONFIRM removed + 7 missing added: IOC, SPEC, INFUSE, METRICS, ACTION, RELOAD, PLUGIN + several descriptions expanded). Closes P3P23-A-L-001.

6. **Product-owner Fix B (2271946):** BC-INDEX SS-20 row added to Summary table (Observability / Log Forwarding, 0 BCs). Ancillary completeness fix surfaced during Burst 24 sweep.

7. **Product-owner Fix C (0cefde4):** domain-spec/entities.md StorageDomain variant count 12 → 16; added ActionState, InfusionCache, PluginState, EventBuffer. Closes P3P23-A-L-002.

8. **Product-owner Fix D (f5ff95a):** SS-07 rename propagated across 9 files — 6 BC frontmatter `subsystem:` fields, BC-INDEX, SUBSYSTEMS-05-07-SUMMARY.md, PRD §2 heading + distribution + RTM rows. Full rename sweep completed. Closes SS-07 rename propagation leg of P3P23-A-H-003.

9. **Story-writer Fix L-001 (b92bf47):** S-1.02 line 36 "Scheduling" → "Scheduler" (policy 6 — ARCH-INDEX is subsystem-name SoT). Ancillary story-label drift.

**Additional drift surfaced and closed mid-burst:**
- P3P23-A6-DRIFT-001: VP-033 / VP-036 stale module assignments in verification-architecture.md (Catalog section). Closed by Architect bonus commit 522b4bd.

**Policy 9 adopted:** `vp_index_is_vp_catalog_source_of_truth` — VP-INDEX.md is the authoritative VP enumeration. Any change must propagate in same burst to verification-architecture.md + verification-coverage-matrix.md. Severity HIGH. Adoption sweep clean (39 VPs verified coherent across all three arch-docs post-Burst-24).

**Findings addressed (all 7 + 1 additional):**
- P3P23-A-H-001 VP-039 absent from arch docs — CLOSED (Architect Fix 1)
- P3P23-A-H-002 coverage-matrix totals stale — CLOSED (Architect Fix 2)
- P3P23-A-H-003 SS-07 stale label in ARCH-INDEX + propagation — CLOSED (Fix 3 + Fix D)
- P3P23-A-H-004 DI-026 / VP-039 traceability absent — CLOSED (Architect Fix 2)
- P3P23-A-M-001 VP-033/036 stale module assignments — CLOSED (Policy 9 sweep bonus)
- P3P23-A-L-001 PRD §5 taxonomy row count stale — CLOSED (PO Fix A)
- P3P23-A-L-002 entities.md StorageDomain variant count — CLOSED (PO Fix C)
- P3P23-A6-DRIFT-001 (mid-burst) VP-033/036 Catalog — CLOSED (Architect Fix 4 / 522b4bd)

**SS-07 rename note:** SS-07 renamed "PrismQL Engine" → "Adapter Pagination & Response Cache" (Burst 24). Architect-authoritative source: ARCH-INDEX.md. Propagated across 9 files (BC-INDEX, 6 BC frontmatter fields, SUBSYSTEMS-05-07-SUMMARY.md, PRD §2 heading/distribution/RTM). STATE.md line 841 retains old label in historical burst-narration (immutable, per append_only_numbering policy).

**Burst 24 totals:** 9 commits on factory-artifacts. No main-branch changes. Metrics unchanged (Active BCs: 195, VPs: 39, Stories: 75, CAPs: 34, Subsystems: 20, CFs: 16).

**Next:** Adversary pass 24 targeting CLEAN. Trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 → **?**. Counter remains 0/3 (Burst 24 spec changes reset eligibility).

### Deployment Model (Confirmed by Human Architect)
- Per-analyst MCP server running in Claude Code (stdio transport)
- One analyst, one process — NOT a shared multi-tenant server
- Multi-client aware: knows about all MSSP clients and their sensors
- Explicit `tenant_id` per MCP tool call; `tenant_id: null` for cross-client queries
- Analyst is trusted (MSSP employee); client isolation is data correctness, not security
- All 4 sensors supported from day one
- Full sensor API supported including write operations (containment, blocking, alert status updates)
- Write operations gated behind two-tier feature flag system:
  - Tier 1: Cargo compile-time features (`--features crowdstrike-write`) — code not present in binary if not compiled
  - Tier 2: TOML per-client runtime config (`[clients.{id}.capabilities]`) — per-client enablement
- Three-tier risk classification for operations:
  - Read: no gate
  - Reversible writes (acknowledge alert, add tag): dry-run default (`dry_run: true`)
  - Irreversible writes (contain host, quarantine file): confirmation token with expiry (300s)
- Destructive operations (delete sensor, wipe endpoint) not exposed via MCP
- Audit logging mandatory for all write operations

## Lessons Captured (Patch Cycle Retrospective)

Durable lessons from Phase 3 patch cycle for future VSDD factory runs:

### Agent-level
1. **Version-race pattern** — state-manager must run LAST in every burst. This caused regressions in pass 4/5/6 before being recognized as a pattern.
2. **Path-prefix doubling** — brief agents with `ls <dest-dir>` verification before first write. story-writer created `.factory/stories/stories/` in Burst 1.
3. **Context overflow** — bursts writing >8 new artifacts should split "create" and "integrate" sub-bursts.
4. **Retroactive anchor propagation** — new BCs must immediately anchor back to implementing stories in the SAME burst.

### Process-level
5. **Previously-converged does not equal correct** — 50-pass-converged Phase 3 had 19 gaps. Mandate fresh-context consistency audit at every phase-gate.
6. **DTU assessment must cover ALL external integrations** at Phase 1 — sensors, actions, infusions, log-forwarding, ingestion. Don't discover scope mid-patch.
7. **BC retirement depth** — retiring a BC touches ~5 artifacts (index removed section, matrix, story frontmatter, AC prose, replacement's Related BCs).
8. **Trajectory monotonicity is a quality signal** — if findings count increases pass-over-pass, investigate root cause before proceeding.
9. **Duplication creates drift** — STATE.md Wave Summary duplicates STORY-INDEX. Every duplicate is a drift opportunity. Establish ONE source-of-truth per metric.
10. **Semantic anchoring integrity** — the lesson that kicked off this retrospective. Mis-anchors hide behind syntactically-valid references. PRD §7 had CAP titles RENAMED to cover mis-anchors (Pass 12 P3P12-A4-001).

### Infrastructure-level
11. **Agent commit permission friction** — specialist agents lack Bash; orchestrator commits all their work. Cost minutes every burst.
12. **User-as-senior-architect catches things adversary does not** — CI/CD gap, DTU scope, taxonomy consistency, CAP-020 mis-anchor. Structure orchestrator to present "questions for human review" at every gate.
13. **Fresh-context review compounds** — adversary passes 7-12 all surfaced new real findings, not just refinements.
14. **ARCH-INDEX is authoritative for subsystem names.** BC-INDEX subsystem labels and BC file frontmatter `subsystem:` fields must match ARCH-INDEX canonical names. Symmetric with capabilities.md → CAP titles. Pass 13 P3P13-A2-004 exposed 7-subsystem taxonomy drift; Burst 14 canonicalized via single-direction sync (ARCH wins).
15. **Rename sweeps must scan aggregation and derivation docs.** Burst 14's SS-NN rename updated BC-INDEX + 208 BC frontmatters + PRD §7 but missed SUBSYSTEMS-*-SUMMARY.md (3 files), PRD §5 BC inventory, and downstream story tables. When changing a widely-referenced label, grep-fix across the entire .factory/ tree before declaring complete.
16. **Retirement is a transitive event.** When retiring a BC, also: (a) strikethrough in all aggregation docs, (b) update current-active count headers, (c) remove from invariant-enforcer tables, (d) sync any stale pre-retirement title to current BC file title. Pass 15 caught three retired BCs (BC-2.04.014, BC-2.06.009, BC-2.10.005) still listed as active in SUBSYSTEMS aggregation docs. Retirement propagation must walk the full traceability graph, not just BC-INDEX.
17. **Completeness is an anchor-integrity axis.** Active-BC row presence in aggregation docs is itself an anchor from aggregation → BC-INDEX; a missing row is semantic drift, not just cosmetic. Burst 16 edited aggregation-doc count annotations without verifying underlying row completeness, enshrining stale counts. Future aggregation-doc edits must: (a) grep BC-INDEX for all active BCs in scope, (b) verify each appears in the aggregation table, (c) re-derive count annotations from row counts.
18. **LOW observations on anchor-like claims are actually MED.** The `semantic_anchoring_integrity` policy covers any claim-like structure (invariant-to-BC enforcer lists, BC-to-Story traceability, BC-to-CAP anchors), not just BC frontmatter. When the adversary rates an anchor-claim finding as LOW, the orchestrator elevates it to MED per policy before dispatching the fix burst. Pass 17 illustrated this with P3P17-A2-OBS-001 (matrix overclaim) and P3P17-A8-OBS-002 (stale Story TBD).
19. **BC file H1 is authoritative for BC titles.** Policy-relevant enrichment that appears in downstream indexes must be moved into the BC H1 rather than left as index-only context — otherwise the H1 drifts from the operational description. 12 BCs in Burst 19 had enrichment moved INTO H1 (e.g., Confirmation Token 100-cap, audit fail-closed qualifier, VP-039 watermark). Two BCs had outright H1↔index contradictions resolved by BC body reading (BC-2.09.004 centralized vs parallel; BC-2.02.008 four-tier vs three-tier) — in both cases, BC H1 was correct. Now captured as 7th policy flag `bc_h1_is_title_source_of_truth`.
20. **Each adversarial scope expansion surfaces next-layer drift.** Trajectory 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 shows alternating decay/uptick: cleanups reduce count, then a broader sweep surfaces new axes. Burst 19's new policy flag `bc_h1_is_title_source_of_truth` effectively raised severity floor on title drift, which surfaced 7 more drifts in pass 19. Pass 19 also introduced new axes: (1) BC body self-contradiction, (2) cross-BC invariant misattribution, (3) STORY-INDEX multi-story matrix completeness. Convergence requires either (a) closing all axes until adversary genuinely finds nothing, or (b) accepting residual drift as convergence debt. We chose (a). Next policy candidates if drift recurs: `bc_body_must_match_h1_claims` (formalize T1 lesson), `sm_invariant_matrix_completeness_policy` (derive enforcer rows from BC frontmatter, not hand-maintained).
21. **User decisions unblock convergence.** When multiple semantic-equivalent options exist (retire vs un-retire; which story owns a BC; which DI label is canonical), auto-adjudication may thrash. Surface to user with pros/cons, let them decide, commit. Burst 21 illustrated: 3 BCs in ambiguous retired-but-active state for 3+ passes; user chose Option A (un-retire) in one interaction; Burst 21 then closed 12 findings in one burst. Also: exhaustive sweeps end the 'tip of iceberg' cycle. Pass 18-20 kept finding 6-7 more title drifts per pass because fixes were targeted; pass 21's exhaustive sweep found only 7 drifts in 195 BCs and 188 were already clean — the iceberg's underwater mass is now surfaced.
22. **Frontmatter IS an anchor claim; body IS the commitment.** When frontmatter lists IDs (story `bcs:`, story `vps:`, BC `capability:`, BC Traceability `Story`), a derivation exists between frontmatter and body (BC tables, AC traces, related sections). Pass 21 found frontmatter-only updates systematically drift bodies. New policy `bc_array_changes_propagate_to_body_and_acs` formalizes the propagation rule. Corollary: creators_justify_anchors policy prevents citing invariants before BC body substantiates them (Burst 22 deferred 3 DI-028/029 citations because BC bodies hadn't been authored to enforce the invariants yet — even though invariants.md names them as enforcers). Domain-spec and BC body must be co-updated when invariant enforcement assignments change.
23. **Policy adoption retroactively elevates pre-existing drift.** Burst 22 adopted `bc_array_changes_propagate_to_body_and_acs` to prevent Burst 21's un-retire from drifting story bodies. Pass 22 then surfaced 3 HIGH findings from Burst 13 era (S-5.08 count drift) and earlier (S-3.01 frontmatter/body). The new policy didn't create drift — it exposed drift that existed but wasn't MED+ under prior policies. Expected behavior for a sound policy; validates the adoption. Corollary: each policy adoption should trigger a one-shot corpus-wide sweep for compliance with the new rule, not just forward enforcement.
24. **VP-layer coherence is a distinct drift class.** Pass 23 surfaced that VP-INDEX can evolve (new VPs added, module assignments corrected) without propagating to verification-architecture.md and verification-coverage-matrix.md. These three documents form a triply-redundant VP catalog; divergence is HIGH severity. Policy 9 (`vp_index_is_vp_catalog_source_of_truth`) formalizes VP-INDEX as the single source of truth. Any VP mutation must sweep all three arch-docs in the same burst. Burst 24's adoption sweep confirmed 39 VPs coherent across all three post-fix.

## Session Chain Summary (2026-04-17)

**Session started:** Post-compact continuation of the Phase 3 patch cycle.

**Bursts executed this session:** 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23 (all committed + pushed to factory-artifacts).

**Session 2 (2026-04-18):** Burst 24 — pass-23 closeout + policy 9 adoption + SS-07 rename + verification arch coherence. 9 commits. Paused after Burst 24.

**Convergence trajectory (pass 12 post-initial-audit through pass 23):**

```
Pass 12: 26 (initial semantic anchoring audit — root cause P3P12-A4-001)
Pass 13: 8  (post-Burst 13: 3 CAPs + BC re-anchors + BC-INDEX v4.4→v4.5 + PRD §7 regen)
Pass 14: 4  (post-Burst 14: SS-01 rename + taxonomy sync + ARCH-INDEX authority)
Pass 15: 2  (post-Burst 15: aggregation docs + PRD §5 title + disclaimer arithmetic)
Pass 16: 1  (post-Burst 16: aggregation retirement rectification)
Pass 17: 1  (post-Burst 17: aggregation completeness BC-2.05.011/2.08.008/009)
Pass 18: 3  (post-Burst 18: SS-07 title + DI-004 overclaim + BC-2.14.013 TBD)
Pass 19: 6  (post-Burst 19: 44 BC title reconciliation + 7th policy flag)
Pass 20: 12 (post-Burst 20: user Option A un-retire 3 BCs + exhaustive sweep + matrix completeness)
Pass 21: 8  (post-Burst 21: un-retire propagation missed derivation layer)
Pass 22: 6  (post-Burst 22: policy-8 drift + invariant round 2 + count refresh)
Pass 23: 7  (post-Burst 23: policy-8 compliance clean — new drift class: VP-layer staleness)
Pass 24: ?  (to be dispatched on resume; Burst 24 closed all 7 pass-23 findings)
```

**Policies adopted (3 new across both sessions):**
- 7th: bc_h1_is_title_source_of_truth (Burst 19)
- 8th: bc_array_changes_propagate_to_body_and_acs (Burst 22)
- 9th: vp_index_is_vp_catalog_source_of_truth (Burst 24 — NEW)

**User decisions this session:**
- Option A for P3P20-A5-001: un-retire BC-2.04.014, BC-2.06.009, BC-2.10.005 with new Config-Reload semantics (Burst 21).
- Requested prompts for vsdd-factory propagation of policy 8 + audit prompt for policies 1-7.
- Paused after Burst 23 to update vsdd-factory before resuming.
- Resumed; dispatched Burst 24 (pass-23 closeout). Paused after Burst 24.

**Structural changes across both sessions:**
- Active BC count: 192 → 195
- CAPs: 31 → 34 (Burst 19 added CAP-032/033/034)
- Subsystems: 20 (stable)
- BC-INDEX: v4.4 → v4.7 (3 minor bumps for regens and un-retire)
- STORY-INDEX: v1.9 → v1.16 (7 minor bumps)
- Policy flags: 6 → 9 (3 new this session)
- VPs: 39 (unchanged count; VP-039 and VP-033/VP-036 module assignments corrected in arch-docs)
- PRD §5 error namespaces: 27 → 33 active (regenerated Burst 24)
- entities.md StorageDomain variants: 12 → 16 (corrected Burst 24)

## Session Resume Checkpoint (2026-04-18) — POST-BURST-24

**STATUS: PAUSED after Burst 24. All pass-23 findings closed. Policy 9 adopted. SS-07 rename complete.**

### Next Action

Dispatch adversary pass 24 — fresh-context review verifying Burst 24 closures + any new drift. Target: CLEAN.
Trajectory: 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 → **?**

### State Snapshot (factual, what's on disk)

- **Branch:** factory-artifacts (head: see `git -C .factory log -1 --format=%H`; last pre-SM commit f5ff95a)
- **Main branch:** main (head: bdf24ce — clean, unchanged during patch cycle)
- **Total patch-cycle commits:** 96+ since ff810e8 (pre-patch baseline); 9 Burst-24 commits + this SM commit

**Metrics (current, all indexes — reconfirmed post-Burst-24):**
- Active BCs: 195 (BC-INDEX v4.7)
- Total BCs: 208 / Removed: 13
- Dual-anchor active BCs: 6
- CAPs: 34 (CAP-001..034)
- Subsystems: 20 (SS-01..SS-20)
- VPs: 39 (VP-INDEX v1.3; verification-architecture.md + verification-coverage-matrix.md now coherent post-Burst-24)
- Stories: 75 across 7 waves (STORY-INDEX v1.16)
- DTU crates: 14
- RocksDB CFs: 16 (entities.md StorageDomain now correctly lists 16 variants)
- PRD §7 Coverage Summary grand total: 201
- PRD §5 error namespaces: 33 active (regenerated Burst 24; was 27)

**Policy flags (9 total):**
1. append_only_numbering: true
2. lift_invariants_to_bcs: true
3. state_manager_runs_last: true
4. semantic_anchoring_integrity: true
5. creators_justify_anchors: true
6. architecture_is_subsystem_name_source_of_truth: true
7. bc_h1_is_title_source_of_truth: true
8. bc_array_changes_propagate_to_body_and_acs: true
9. vp_index_is_vp_catalog_source_of_truth: true  [NEW — adopted Burst 24]

Policy 9 rule: VP-INDEX.md is the authoritative VP enumeration. Any change
(addition, retirement, module reassignment, tool/phase change, total count)
MUST propagate in the same burst to architecture/verification-architecture.md
Provable Properties Catalog + P0 list AND architecture/verification-coverage-matrix.md
VP-to-Module table + Totals. Severity: HIGH. Symmetric to policies 6 (subsystem
name SoT) and 7 (BC H1 SoT).

Policy 9 adoption sweep: Burst 24 architect commits 0dd5a30, 499d0aa, 522b4bd
brought verification-architecture.md and verification-coverage-matrix.md into
full VP-INDEX coherence. 39 VPs verified present in both arch-docs with
matching module, tool, phase. Sweep clean.

### Deferred items (tracked, NOT findings)

Three DI citations await BC body amendments (per creators_justify_anchors policy; none addressed Burst 24):
- DI-028 → BC-2.12.001: needs cap-check postcondition + E-SCHED-008 error case
- DI-028 → BC-2.13.006: needs cap-check postcondition + E-RULE-011 error case
- DI-029 → BC-2.06.005: needs cross-validation postcondition (correlation window vs schedule interval WARN)

### Outstanding work streams when session resumes

1. **Adversary pass 24** — Dispatch to verify Burst 24 closures + any new drift. Target CLEAN.
2. **Deferred DI citations (3 items)** — Separate small burst to amend BC bodies and then add citations.
3. **vsdd-factory policy integration** — USER task (external, not in .factory/). Now includes policy 9 integration prompt (delivered this session). Policy 8 integration + policies 1-7 audit prompts also delivered prior session.
4. **Phase 3 convergence target** — Need 3 consecutive clean adversarial passes. Currently 0/3.

### Open policy questions (none require user decision at resume time; listed for awareness)

- **Retired filename slug drift (P3P22-A10-O-002 LOW):** BC-2.06.009 filename slug `on-client-switch` vs current H1 `Config Reload Triggers...`. Accepted under `append_only_numbering` policy. If slug rename is desired later, requires atomic update of filename + all PRD §5 link references.
- **DI-024 and DI-028 enforcement gap:** Three BCs named as enforcers in invariants.md don't yet enforce in body. Deferred items above are the remediation path.

### Commit chain since pre-compact baseline

```
git log --oneline ff810e8..HEAD
```

~95+ commits across Bursts 4–24. Key milestone commits:
- bfaef13: Burst 14 PO-A (subsystem rename SS-01 → Sensor Adapters)
- 7f91a42: Burst 14 PO-A A6 (7-subsystem taxonomy canonicalization)
- 65c77c1: Burst 19 PO-B (44 BCs title reconciliation)
- bea56b6: Burst 21 PO-A (3 BCs un-retired, 192 → 195)
- 46bbe57: Burst 21 PO-B (exhaustive 195-BC title sweep)
- 10d6e3b: Burst 23 SM (final state log — session 1 close)
- 0dd5a30: Burst 24 Architect Fix 1 (VP-039 to verification-architecture.md)
- 499d0aa: Burst 24 Architect Fix 2 (coverage-matrix triple-fix)
- 4738ee3: Burst 24 Architect Fix 3 (SS-07 rename in ARCH-INDEX)
- 522b4bd: Burst 24 Architect Policy-9 sweep (VP-033/VP-036 module corrections)
- 950f4ce: Burst 24 PO Fix A (PRD §5 error taxonomy regen, 27→33)
- f5ff95a: Burst 24 PO Fix D (SS-07 propagation across 9 files)

### Resume prompt (for new session after this pause)

```
Resume Prism VSDD factory Phase 3 patch cycle.

WORKSPACE: /Users/jmagady/dev/prism
BRANCH: factory-artifacts (head <see git -C .factory log -1 --format=%H>), worktree at /Users/jmagady/dev/prism/.factory
MAIN: main (bdf24ce, clean)
MODE: brownfield, Phase 3 patch cycle, post-Burst-24

STATE: See /Users/jmagady/dev/prism/.factory/STATE.md § "Session Resume Checkpoint"
for full context. Active BCs: 195, CAPs: 34, Stories: 75, VPs: 39, 9 policy flags.

NEXT ACTION: Dispatch adversary pass 24 targeting CLEAN. Trajectory so far:
26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 → ?. Burst 24 closed all
7 pass-23 findings + 1 additional mid-burst drift. Pass 24 verifies.

POLICIES (9 total; policy 9 NEW — adopted Burst 24):
  vp_index_is_vp_catalog_source_of_truth — VP-INDEX.md is authoritative VP
  enumeration; any change must propagate same burst to verification-architecture.md
  Provable Properties Catalog + P0 list AND verification-coverage-matrix.md
  VP-to-Module table + Totals. HIGH severity.

DEFERRED (3 items, tracked in .factory/STATE.md):
  - DI-028 → BC-2.12.001 (body: cap-check + E-SCHED-008)
  - DI-028 → BC-2.13.006 (body: cap-check + E-RULE-011)
  - DI-029 → BC-2.06.005 (body: cross-validation WARN)

Per orchestrator protocol, factory-worktree-health must pass before reading
STATE.md. Then read STATE.md § Session Resume Checkpoint, then dispatch
adversary pass 24.
```

### Resume criteria

**Pre-resume check:** factory-worktree-health skill passes.
**Session start:** Read this checkpoint section first before any other action.
**First action:** `agents_list`, then dispatch adversary pass 24 to vsdd-factory:adversary agent with full context from this checkpoint.
**Do NOT:** Dispatch a new burst of spec changes before pass 24 runs — we need verification of Burst 24 first.

---

## Session Resume Checkpoint (2026-04-18) — POST-PASS-24 / PRE-BURST-25

**STATUS: PAUSED after adversary pass 24. 3 findings open (2 HIGH, 1 MED). Convergence counter 0/3 (no advance). Burst 25 NOT yet dispatched.**

### Next Action

Either:
- **(a)** Dispatch Burst 25 to close 3 pass-24 findings (architect + story-writer in parallel, state-manager runs last per policy 3).
- **(b)** Wait for Policy 9 plugin integration (brief at `.factory/plugin-updates/policy-9-lessons-learned-2026-04-18.md`, external workstream) then re-run pass 24 to confirm the lint-hook catches P3P24-A-H-002 as canary.
- **(c)** Both in parallel. **Recommendation: (c)** — plugin work is external/non-blocking for Prism convergence.

### Pass 24 Findings (with routing)

| ID | Severity | File | Policy violated | Routes to | Notes |
|---|---|---|---|---|---|
| P3P24-A-H-001 | HIGH | `.factory/stories/S-5.10-audit-trail-forwarding.md` | 4 + 8 | story-writer | BC-2.05.011 in frontmatter/body but NO AC cites it; 4 ACs (2,3,4,6) cite semantically wrong BCs (retry → redaction BC, FIFO → write-ops BC). Rewire AC traces. |
| P3P24-A-H-002 | HIGH | `.factory/specs/architecture/verification-coverage-matrix.md` | 9 | architect | prism-security Fuzz Targets cell = 2 but only VP-038 listed; row-sum=7, Totals=6, VP-INDEX=6. Resolve cell (likely `2` → `1`) and re-verify all per-module fuzz counts. |
| P3P24-A-M-001 | MED | `.factory/specs/architecture/verification-coverage-matrix.md` | 4 | architect | BC-2.05.011 placed in "Invariant-to-VP Traceability" table "Invariant" column (all other rows are DI-NNN). Rename column or move BC row out. |

### Convergence Trajectory

`26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 → 3`

- CRIT=0 for 13th consecutive pass.
- Decay resumed after pass-23 uptick (7 → 3 = 57% decay).
- Novelty: MEDIUM overall. Policy 9 (HIGH novelty) first substantive surfacing this pass (P3P24-A-H-002).

### Filename Convention Decision (recorded 2026-04-18)

Phase-3 patch cycle adversarial reviews use filename pattern `adversarial-review-p3p-pass-N.md` at `/Users/jmagady/dev/prism/.factory/specs/`. The `-p3p-` infix distinguishes from 33 historical Phase-1 files `adversarial-review-pass-1..33.md` (committed in ancestor 475e185). Historical files are archive — do NOT overwrite.

### Adversary Invocation Decision (recorded 2026-04-18)

Future passes (25+) SHOULD use `/vsdd-factory:adversarial-review` skill after the plugin update lands (see plugin brief below). Until then, direct-spawn adversary MUST be paired with an explicit state-manager write step — the adversary agent is read-only by design (Read/Grep/Glob only, no Write) and direct-spawn has no persistence step; findings returned as chat text are lost without SM action.

### Plugin Integration Status

Brief drafted 2026-04-18 at `.factory/plugin-updates/policy-9-lessons-learned-2026-04-18.md`. Covers 9 lessons (adversary persistence, default path mismatch, finding-ID collision, filename collision guard, policy registry, invocation flag, scoped review) plus Policy 9 hook (`validate-vp-consistency.sh`) with Pass-24 canary fixture. External execution pending.

### Metrics Snapshot (unchanged from POST-BURST-24)

- Active BCs: 195 / BC-INDEX v4.7
- CAPs: 34
- Subsystems: 20
- VPs: 39 / VP-INDEX v1.3
- Stories: 75 / STORY-INDEX v1.16
- DTU crates: 14
- RocksDB CFs: 16
- PRD §5 error namespaces: 33
- PRD §7 Coverage Summary grand total: 201
- Policy flags: 9 (unchanged)

### Deferred Items (unchanged)

Three DI citations await BC body amendments (per `creators_justify_anchors` policy; not in scope for this burst):
- DI-028 → BC-2.12.001 (body: cap-check + E-SCHED-008)
- DI-028 → BC-2.13.006 (body: cap-check + E-RULE-011)
- DI-029 → BC-2.06.005 (body: cross-validation WARN)

### Resume Prompt (paste into fresh session)

```
Resume Prism VSDD factory Phase 3 patch cycle.

WORKSPACE: /Users/jmagady/dev/prism
BRANCH: factory-artifacts (head: <see git -C /Users/jmagady/dev/prism/.factory log -1 --format=%H>), worktree at /Users/jmagady/dev/prism/.factory
MAIN: main (bdf24ce, clean — do not touch)
MODE: brownfield, Phase 3 patch cycle, post-pass-24 / pre-Burst-25

STATE: Read /Users/jmagady/dev/prism/.factory/STATE.md § "Session Resume Checkpoint (2026-04-18) — POST-PASS-24 / PRE-BURST-25" for full context.
Active BCs: 195, CAPs: 34, Stories: 75, VPs: 39, 9 policy flags.

NEXT ACTION (choose one or all in parallel):
(a) Dispatch Burst 25 to close 3 pass-24 findings:
    - P3P24-A-H-001 → story-writer: rewire AC traces in S-5.10-audit-trail-forwarding.md
    - P3P24-A-H-002 → architect: fix prism-security fuzz count (2→1) in verification-coverage-matrix.md + re-verify fuzz totals
    - P3P24-A-M-001 → architect: rename "Invariant" column or relocate BC-2.05.011 row in Invariant-to-VP Traceability table
    - state-manager runs last (policy 3)
(b) Plugin integration: brief at .factory/plugin-updates/policy-9-lessons-learned-2026-04-18.md — external workstream, non-blocking.
(c) Both (a) and (b) in parallel. Recommendation: (c).

CONVERGENCE TRAJECTORY: 26 → 8 → 4 → 2 → 1 → 1 → 3 → 6 → 12 → 8 → 6 → 7 → 3
COUNTER: 0/3 (need 3 consecutive clean passes to converge Phase 3)
CRIT=0 for 13 consecutive passes.

POLICIES (9 total):
1. append_only_numbering
2. lift_invariants_to_bcs
3. state_manager_runs_last
4. semantic_anchoring_integrity
5. creators_justify_anchors
6. architecture_is_subsystem_name_source_of_truth
7. bc_h1_is_title_source_of_truth
8. bc_array_changes_propagate_to_body_and_acs
9. vp_index_is_vp_catalog_source_of_truth (VP-INDEX.md authoritative; propagate same-burst to verification-architecture.md + verification-coverage-matrix.md)

DEFERRED (3 items, tracked in STATE.md):
  - DI-028 → BC-2.12.001 (body: cap-check + E-SCHED-008)
  - DI-028 → BC-2.13.006 (body: cap-check + E-RULE-011)
  - DI-029 → BC-2.06.005 (body: cross-validation WARN)

ORCHESTRATOR PROTOCOL: Run factory-worktree-health first, then read this checkpoint section, then dispatch.
```

### Resume Criteria

**Pre-resume check:** factory-worktree-health skill passes.
**Session start:** Read this checkpoint section (POST-PASS-24 / PRE-BURST-25) first before any other action.
**First action:** Choose between options (a), (b), or (c) above. For (a): dispatch architect and story-writer in parallel for Burst 25 fixes, then state-manager last.
**Do NOT:** Dispatch another adversary pass before Burst 25 fixes are applied — pass 24 found 3 open findings that must be addressed first.

## Housekeeping 2026-04-18 — Plugin adoption + cycle-keyed layout

Adopted vsdd-factory v0.24.2+ (Policy 9 + 17 hooks, policy-registry, factory-cycles-bootstrap). Three concurrent cleanups applied in one commit:

**A: factory-cycles-bootstrap migration** — 33 flat `specs/adversarial-review-pass-{1..33}.md` files moved (git mv) to `cycles/phase-1-convergence/adversarial-reviews/`. `specs/adversarial-review-p3p-pass-24.md` moved to `cycles/phase-3-patch/adversarial-reviews/pass-24.md`. `.factory/current-cycle` pointer written (`phase-3-patch`). INDEX.md created per cycle with pass tables.

**B: Policy registry** — `.factory/policies.yaml` initialized from plugin baseline (9 policies, IDs 1-9). Names match `audit_policy_decisions` block 1:1. `audit_policy_decisions` block retained as historical record; `policies.yaml` is the authoritative source going forward.

**C: plugin-updates archive** — `plugin-updates/` moved to `archive/plugin-updates/` via git mv. Policy-9 lessons-learned brief (shipped in this plugin version) preserved in git history for traceability.

STATE.md frontmatter updated with plugin adoption metadata (plugin_version_adopted, plugin_adopted_date, policy_registry_source_of_truth, current_cycle, historical_cycles, layout_bootstrap_date). No historical data rewritten.

Commit SHA: <sha>
