---
project: prism
mode: brownfield
phase: 2-architecture
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
current_step: "Phase 2 architecture created — awaiting adversarial review"
awaiting: "adversarial review of architecture + spec package"
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
- [ ] **NEXT**: Adversarial review of architecture before story decomposition

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
