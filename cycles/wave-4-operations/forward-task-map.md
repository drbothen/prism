# Forward Task Map — Prism Pipeline (sealed at D-444 PRE-COMPACT 2026-05-12)

**Authority:** This file is the canonical forward roadmap. Post-compact sessions read this file for ANY context beyond the immediate B→C→A sequence in SESSION-HANDOFF successor_focus.

**Methodology:** Built from systematic reads of STORY-INDEX v2.65, story specs for dependency resolution, ADR-023 v1.18, tech-debt-register v2.15, and project memory vision elements. Cycle resolution documented in TIER 3.

---

## TIER 1 — Immediate (in SESSION-HANDOFF v7.178 successor_focus)

User's locked-in B → C → A sequence. See SESSION-HANDOFF.md `successor_focus` for full dispatch specs.

- STEP 1 (B): /vsdd-factory:sprint-review on PREREQ trio (A=#142, B=#143, C=#144)
- STEP 2 (C): Maintenance burst on deferred items (priority list: F-LP4-OBS-001, TD-S-PLUGIN-PREREQ-C-001, TD-VSDD-091 sweep)
- STEP 3 (A): PLUGIN-MIGRATION Wave 1 dispatch — but FIRST gated on TIER 2

---

## TIER 2 — Plugin Migration Wave 0 prerequisite gap closure (BLOCKING TIER 3)

**Status:** Both [planned], must merge before any Wave 1 story. PREREQ-D and PREREQ-E can run serially (D first — boot wiring is architecturally upstream) or in parallel.

| Story | Scope | depends_on | Notes |
|-------|-------|------------|-------|
| S-PLUGIN-PREREQ-D | prism-bin/prism-spec-engine: Wire PluginRuntime into boot step 7; .prx Build/Sign/Load Pipeline (unsigned v1.0; boot warning + audit log; signing deferred TD-PLUGIN-SIGNING-001) | PREREQ-F + PREREQ-A — READY | TD-B-002/005/011/012 routed here from carry-forward matrix |
| S-PLUGIN-PREREQ-E | prism-sensors/prism-spec-engine: Un-seal SensorAuth + Deprecate/Remove CustomAdapter Rust Trait + migrate spec_parser.rs call sites to PluginRegistry | PREREQ-F + PREREQ-A — READY | TD-A-003 (WriteToolInvalidationMap extensibility) routed here |

---

## TIER 3 — PLUGIN-MIGRATION Wave 1 (5 stories)

### Cycle resolution finding (D-444)

STORY-INDEX v2.65 rows show:
- PLUGIN-MIGRATION-001-A depends_on: PREREQ-A + PREREQ-B + PREREQ-C + PREREQ-E + PLUGIN-MIGRATION-001-D + PLUGIN-MIGRATION-001-E
- PLUGIN-MIGRATION-001-D depends_on: PREREQ-B + PREREQ-C + PLUGIN-MIGRATION-001-A **(STALE ENTRY)**
- PLUGIN-MIGRATION-001-E depends_on: PREREQ-D + PLUGIN-MIGRATION-001-D

This creates an apparent cycle: 001-A → 001-D → 001-A. **Resolution per D-334 changelog:** "PLUGIN-MIGRATION-001-A depends_on updated to include PLUGIN-MIGRATION-001-D and PLUGIN-MIGRATION-001-E (replacement-before-deletion per user decision on HIGH-007)." The design intent is 001-D and 001-E must exist BEFORE 001-A can delete the old code. The stale entry is in 001-D's depends_on: "PLUGIN-MIGRATION-001-A" should not be there — 001-D authors the new TOML specs, it does not depend on the deletion story (001-A). This is a **STORY-INDEX data error**.

**Action required before TIER 3 dispatch:** Fix STORY-INDEX row for 001-D — remove PLUGIN-MIGRATION-001-A from its depends_on. This is a story-writer data fix, not a design change.

### True topological order — Wave 1

1. S-PLUGIN-PREREQ-D (boot wiring — blocks 001-C, 001-E)
2. PLUGIN-MIGRATION-001-D (author 4 production TOML sensor specs + DTU-parity tests — blocks 001-E, 001-A)
3. PLUGIN-MIGRATION-001-E (CrowdStrike OAuth2 refresh as .prx WASM plugin — blocks 001-A)
4. PLUGIN-MIGRATION-001-A (delete 4 named auth modules + replace init_registry_for_org — blocks 001-B, 001-C)
5. PLUGIN-MIGRATION-001-B + PLUGIN-MIGRATION-001-C in parallel (001-B: prism-query dispatch sites; 001-C: prism-ocsf SpecDrivenMapper)

### Scope summary

| Story | Crates | Scope |
|-------|--------|-------|
| PLUGIN-MIGRATION-001-D | prism-sensors, prism-spec-engine | Author 4 production TOML sensor specs (CrowdStrike, Cyberint, Claroty, Armis); reverse-engineer from existing auth modules; DTU-parity tests |
| PLUGIN-MIGRATION-001-E | prism-spec-engine | CrowdStrike OAuth2 Refresh-on-401 as in-repo .prx WASM plugin |
| PLUGIN-MIGRATION-001-A | prism-sensors, prism-bin | Delete 4 named auth modules + re-exports + replace init_registry_for_org; gated on VP-PLUGIN-003 parity test pass |
| PLUGIN-MIGRATION-001-B | prism-query | Convert 5 sensor-name dispatch sites to spec-catalog lookup |
| PLUGIN-MIGRATION-001-C | prism-ocsf, prism-spec-engine | Merge 4 hardcoded mappers → SpecDrivenMapper + .prx WASM transformers |

---

## TIER 4 — PLUGIN-MIGRATION Wave 2 (3 stories)

All 3 depend on Wave 1 completion. Can be sequenced serially or with F/G in parallel after H.

| Story | Crates | depends_on | Scope |
|-------|--------|------------|-------|
| PLUGIN-MIGRATION-001-H | .factory | 001-A | Mark S-2.06/S-2.07/W3-FIX-S307-001/S-3.1.06-ImplPhase superseded |
| PLUGIN-MIGRATION-001-F | prism-sensors, prism-query | 001-A, 001-B | Rewrite 10+ sensor-named test files + compile-fail perimeter test |
| PLUGIN-MIGRATION-001-G | .factory | 001-A, 001-B, 001-C | Doc/ADR/BC sweep — generalize sensor-named architecture docs; fully amends pending BCs BC-2.01.005/006/007/008 + BC-2.02.003/004/005/006 per ADR-023 |

---

## TIER 5 — Unblocked by plugin migration

### Story-level unblocks (after Wave 1 merges)

| Story | Current Status | Unblock condition |
|-------|---------------|-------------------|
| S-5.01-FOLLOWUP-MCP-BOOT | BLOCKED | After 001-A merges — MCP boot step 8 AdapterRegistry assertion (TD-S-PLUGIN-PREREQ-A-004) |
| S-1.12-FOLLOWUP (notify-watcher) | BLOCKED | After 001-A merges |
| S-1.14-REDO (infusion engine) | BLOCKED | After 001-A merges |
| W3-FIX-S307-001 | SUPERSEDED | Closed via 001-A + 001-H |
| W3-FIX-S307-002 | SUPERSEDED | Closed via 001-A + 001-B + 001-H |

**S-3.09 Query Profiling resumption:** Worktree .worktrees/S-3.09 is FROZEN at HEAD 43c41389 per D-298/D-299. BUG-S309-PLUGIN P0 blocked it (migrate 4 built-in adapters to spec-engine). PLUGIN-MIGRATION-001-A/B/C close this bug. After Wave 1 completes, S-3.09 resumes with existing deferred fix-burst-2/3/4 work and requires a fresh LOCAL adversary pass-2 dispatch.

**TD unblocks from Wave 1:**
- TD-S-PLUGIN-PREREQ-A-002 P1: sentinel-nil OrgId in WriteDispatcher — depends on W3-FIX-S307-002 unblock (closed by 001-A/B supersession)
- TD-A-005 P3: EXPLAIN silent-skip UX inconsistency — routed to PLUGIN-MIGRATION-001-B

---

## TIER 6 — Multi-Tenant Wave 3 (37 stories, all MERGED)

All Wave 3 stories are already merged. This tier is complete. Listed for orientation only.

| Epic | Stories | Status |
|------|---------|--------|
| E-3.0: Pre-Wave quick fixes | S-3.0.01 + S-3.0.02 | ALL MERGED |
| E-3.1: OrgId/OrgSlug split | S-3.1.01–07 + S-3.1.06-ImplPhase (8 stories) | ALL MERGED |
| E-3.2: Multi-Tenant DTU state | S-3.2.01–08 (8 stories) | ALL MERGED |
| E-3.3: Customer config + harness | S-3.3.01–06 (6 stories) | ALL MERGED |
| E-3.4: Test migration to harness | S-3.4.01–05 (5 stories) | ALL MERGED |
| E-3.5: DevX fixes (Wave 3.1–3.4) | S-3.5.01 + W3-FIX-WIN + W3-FIX-LEFTHOOK + W3-FIX-CI + W3-FIX-SEC-001/002/003/004/005 + W3-FIX-CODE-001/002/003/004/005/006 + W3-FIX-CREDS-001 (16 stories) | ALL MERGED |
| E-3.6: Holdout refresh | S-3.6.01 + S-3.6.02 (2 stories) | ALL MERGED |
| E-3.7: Multi-Tenant data gen | S-3.7.00–05 (6 stories) | ALL MERGED |

**E-CLEANUP-02 Runtime Reality stories still [planned/blocked]:** S-WAVE5-PREP-01 + S-3.02-FOLLOWUP-RUNTIME (merged PR #141) + W3-FIX-S307-001/002 (superseded by plugin migration) + S-1.12-FOLLOWUP + S-1.14-REDO + S-5.01-FOLLOWUP-MCP-BOOT. These are TIER 5 unblocks, not Wave 3 stories.

---

## TIER 7 — Wave 4+ operational hardening

From tech-debt-register v2.15, Wave 4+ capability TDs filed at Phase 3.A approval gate (D-136):

| TD ID | Priority | Capability | Target |
|-------|----------|-----------|--------|
| TD-W4-AUDIT-QUERY-REPLAY-001 | P2 | Audit Query and Replay — forensic investigation, regulatory audit response, ML training corpus, debug replay | Wave 4+ |
| TD-W4-LOG-FORWARDING-001 | P2 | Outbound log/audit/event forwarding to external sinks per-org (Splunk/Sentinel/Elastic, MSSP centralized, compliance archive) | Wave 4+ |
| TD-W4-ALERTING-WORKFLOWS-001 | P2 | Detection rule engine + alert routing + escalation policies + notification fan-out per-org | Wave 4+ |
| TD-W3-TIMING-001 | P2 | BC-3.5.001/002 wall-clock budget tests (#[ignore] — fragility fix or Criterion migration) | Wave 3.3 or Wave 4 |
| TD-W3-QUOTA-SOAK-001 | P3 | Cross-tenant API quota soak test (60s Tenant A high-frequency + Tenant B unaffected assertion) | Wave 4 planning |
| TD-S-1.07-01 | P1 | CRUD store production wire-up to KeyringBackend — DO NOT CLOSE until Wave 5 prism-mcp lands | Wave 5 |

These represent the operational hardening layer. They are likely to materialize as story epics post-Wave-3, with their own spec crystallization, story decomposition, and wave-gate pipeline.

---

## TIER 8 — End-product convergence (per project memory)

The ultimate target per Prism vision:

- **Rust MCP server for MSSP sensor management** — per-analyst deployment in Claude Code (stdio transport, prism-mcp crate not yet created — Wave 5 gate per TD-S-1.07-01)
- **Multi-tenant aware** — multi-client MSSP model; OrgId/OrgSlug wiring complete in Wave 3
- **All 4 production sensors as TOML plugins**: CrowdStrike, Cyberint, Claroty, Armis (delivered by PLUGIN-MIGRATION-001-D)
- **Ephemeral federated query engine**: PrismQL parser (Chumsky) + DataFusion materialization; query engine execution pipeline complete (PR #141); Layer 1+2 capability gates wired
- **OCSF + protobuf normalization**: SpecDrivenMapper (PLUGIN-MIGRATION-001-C) + .prx WASM transformers
- **RocksDB persistence**: 12 column families, osquery-informed patterns (S-2.01 foundation shipped)
- **Memory budget**: 512MB process / 200MB per-query — specified in NFR catalog; GreedyMemoryPool wired (PR #141)
- **AI-opaque credentials**: CLI/env/vault references; never transit AI context; credential model complete from Wave 1/2
- **Write API behind feature flags**: full sensor API including writes — Wave 4+ stories (W3-FIX-S307-001/002 superseded by plugin migration; WriteExecutor Phase 3 deferred to TIER 7)
- **Quality**: "Best in class, no scope compromises" — no defer-with-surfacing without substantial justification; Standing Rule 3 active

---

## Dependency-chain summary

```
Tier 1 (B → C → A locked in successor_focus)
  |
  v
Tier 2: PREREQ-D + PREREQ-E (Wave 0 closure — both [planned])
  |
  v
Tier 3: PLUGIN-MIGRATION Wave 1 (5 stories in topological order:
         PREREQ-D → 001-D → 001-E → 001-A → [001-B || 001-C])
  |
  v
Tier 4: PLUGIN-MIGRATION Wave 2 (3 stories: 001-H → [001-F || 001-G])
  |
  v
Tier 5: Bundle B Phase B-2 unblocks (S-5.01-FOLLOWUP + S-1.12-FOLLOWUP + S-1.14-REDO)
       + S-3.09 resumption (FROZEN worktree thaws)
  |
  v
Tier 6: Multi-Tenant Wave 3 — ALREADY COMPLETE (37 stories merged)
  |
  v
Tier 7: Wave 4+ operational hardening (alerting, audit replay, log forwarding, timing TD)
  |
  v
Tier 8: End-product convergence (production MSSP MCP, prism-mcp crate, Wave 5)
```

---

## Resume protocol (post-compact)

1. Read `.factory/SESSION-HANDOFF.md` for immediate dispatch (TIER 1 STEP 1: sprint-review).
2. Read this file for any context beyond immediate.
3. Before dispatching first Wave 1 story: fix STORY-INDEX 001-D depends_on (remove stale PLUGIN-MIGRATION-001-A entry). This is a story-writer data fix.
4. Confirm PREREQ-D and PREREQ-E status before dispatching PLUGIN-MIGRATION-001-D (they gate it).
5. Re-read TIER 5 unblock conditions after each Wave 1 merge — S-3.09 worktree thaw particularly depends on 001-A.
6. Wave 4 work begins only after Wave 2 closure; refer to tech-debt-register v2.15 for TD IDs.

---

## Pin block (at D-444 seal)

- develop@ea958a4d (S-PLUGIN-PREREQ-C PR #144, 2026-05-12T23:14:05Z)
- factory-artifacts HEAD: run `git -C .factory log -1 --format='%H'` after this commit
- STORY-INDEX v2.65 (150 stories; 14 PLUGIN-MIGRATION [planned])
- BC-INDEX v4.61 (235 BCs)
- tech-debt-register v2.15 (91 active TDs)
- PREREQ trio (A+B+C) all merged; PREREQ-D/E [planned]
- 4 retained worktrees: S-PLUGIN-PREREQ-C, S-PLUGIN-PREREQ-B, W3-FIX-S307-001, S-3.09 (FROZEN)
