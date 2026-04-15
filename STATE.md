---
project: prism
mode: brownfield
phase: 2-architecture-converged
status: awaiting_gate
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
current_step: "Phase 2 architecture converged — 27 adversarial passes, ready for story decomposition"
awaiting: "human approval to proceed to Phase 3 (story decomposition)"
phase_2_converged: 2026-04-15
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
- [ ] **NEXT**: Phase 3 — Story decomposition

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
- BC-2.14.012: acknowledge_alert STUB (tool schema exists, BC not yet written)
- Auto-case-creation from high-severity rules: noted in CAP-022, needs dedicated BC during story decomposition

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
