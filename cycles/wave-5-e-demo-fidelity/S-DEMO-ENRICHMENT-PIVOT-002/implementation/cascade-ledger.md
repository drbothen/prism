# S-DEMO-ENRICHMENT-PIVOT-002 LOCAL Cascade Ledger

Story: S-DEMO-ENRICHMENT-PIVOT-002 ThreatIntel + NVD Infusion Specs and Plugins
Cycle: wave-5-e-demo-fidelity
Streak target: 3-CLEAN (strict per BC-5.39.001 + D-779)

## Architecture Pivot (D-1211 2026-06-17)

ADR-040 v2.0 ratified DUAL-PATH architecture:
- InfusionType::HttpLookup — declarative TOML, reuses pipeline.rs, for NVD (simple HTTP enrichment)
- InfusionType::Plugin (WASM .prx) — for ThreatIntel (polymorphic ip/domain/hash)

NVD plugin crate (prism-nvd-infusion) to be REMOVED from scope.
Story re-scoped to v1.3: 32 red_gate_tests, 13 pts.
PENDING: PO to add http_lookup to BC-2.19.001 E-INFUSE-004 valid-types before TDD starts.

## Cascade History

### Pass 1 (2026-06-17) — FINDINGS (pre-pivot)

| ID | Severity | Summary |
|----|----------|---------|
| F-001 | CRIT | WASM hollow — enrich_single returns Ok(None) unconditionally |
| F-002 | HIGH | validate_plugin_path dead code — path not checked |
| F-003 | MED | Test tautology — tests verify stubs not behavior |
| F-004 | MED | Test tautology — integration test covers no real path |
| F-005 | LOW | Path traversal insufficient sanitization |

→ Dual-path architecture pivot triggered. Story re-scoped v1.2→v1.3. Not yet re-implemented.
Streak 0/3.

### PENDING before TDD

1. PO: add http_lookup to BC-2.19.001 E-INFUSE-004 valid-types
2. Implementer: remove prism-nvd-infusion WASM crate (use HttpLookup declarative TOML instead)
3. Implementer: wire ThreatIntel as WASM Plugin per ADR-040 v2.0 D2/D3 lift strategy
