---
project: prism
mode: brownfield
phase: 0-ingestion
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
current_step: "Phase 0 gate — awaiting human approval"
awaiting: human_approval
deployment_model: per-analyst-stdio
---

# VSDD Pipeline State — Prism

## Current Phase: 0 — Codebase Ingestion (Brownfield)

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
- [ ] Phase 0 gate (human approval)

### Deployment Model (Confirmed by Human Architect)
- Per-analyst MCP server running in Claude Code (stdio transport)
- One analyst, one process — NOT a shared multi-tenant server
- Multi-client aware: knows about all MSSP clients and their sensors
- Explicit `tenant_id` per MCP tool call; `tenant_id: null` for cross-client queries
- Analyst is trusted (MSSP employee); client isolation is data correctness, not security
- All 4 sensors supported from day one
- Write operations (containment, blocking) excluded from initial scope
