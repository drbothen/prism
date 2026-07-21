---
document_type: planning
created: 2026-07-21
author: state-manager
origin: human directive 2026-07-21 / PG-ARCHITECT-CLAUDEMD-CONTAMINATION-001
status: PARKED — feature-ordered AFTER Wave-A / test-soc findings remediation
---

# File-Size Decomposition Plan (2026-07-21)

## Background

On 2026-07-21, the human adjudicated a production-grade file-size governance stance for
prism (D-1899), superseding the prior passive stance. The governing event was
PG-ARCHITECT-CLAUDEMD-CONTAMINATION-001: a reused architect agent cross-contaminated
from a different project (ferrochain), inserting false file-size rules into CLAUDE.md.
That contamination was remediated; this document records the prism-accurate stance.

**Current governance:** No CI gate today = tracked debt (TD-DECOMP-RATCHET-001 P2).
Soft guideline: ~800 production lines. Proposed ratchet: 1,500-line hard cap for NEW
code, with a day-1 allowlist of the 12 existing giants. Oversized files = scheduled
decomposition debt, not cohesion exceptions.

**Scheduling constraint:** ALL stories in this plan are feature-ordered AFTER Wave-A
(ADR-053/054 native declarative HTTP auth) and test-soc findings remediation.
The ratchet-gate story (S-DECOMP-RATCHET-GATE) lands first; it initializes the
`.factory/file-size-allowlist.toml` and wires `just check-file-sizes` + CI check.
Full story materialization is deferred until the epic is scheduled.

---

## Epic Tree

```
TD-DECOMP-EPIC-001 (P3 umbrella)
│
├── S-DECOMP-RATCHET-GATE  ← FIRST (P2; initializes gate + allowlist)
│   Deliverables: just check-file-sizes (tokei), CI check, file-size-allowlist.toml
│   pre-seeded with the 12 giants. Calibration TBD: total lines vs production lines
│   (engine.rs is 17,041 total but ~4,900 production; tokei strip of #[cfg(test)]?).
│
├── S-DECOMP-ENGINE-A      ← test extraction (low-risk; ~12,100 inline test lines)
├── S-DECOMP-ENGINE-B      ← production module split (plan/execute/cache/planner)
│
├── S-DECOMP-SERVER-A      ← params module extraction
├── S-DECOMP-SERVER-B      ← per-family dispatch split
│
├── S-DECOMP-MATERIALIZATION  ← fan_out / cache / session / resolve
│
├── S-DECOMP-ERROR-MAPPING    ← structured / codes split
│
├── S-DECOMP-PIPELINE         ← request_builder / extraction / normalization
│
├── S-DECOMP-BOOT             ← steps.rs + credential.rs extraction
│
└── S-DECOMP-REMAINING        ← ast.rs, infusion_udf.rs, spec_driven_adapter.rs,
                                  error.rs, filter_parser.rs, prism_describe.rs
                                  (cohesion-grouped; after higher-priority splits)
```

---

## Per-File Split-Boundary Hypotheses

### engine.rs — prism-query (17,041 total / ~4,900 prod)

**Step A (S-DECOMP-ENGINE-A) — test extraction (low-risk):**
~12,100 lines of inline `#[cfg(test)] mod tests` blocks. Extract to
`crates/prism-query/tests/engine_*.rs` files. No production behavior change.
Removes engine.rs from the allowlist day-1.

**Step B (S-DECOMP-ENGINE-B) — production module split:**
Remaining ~4,900 lines. Candidate boundaries:
- `planning/` sub-module: query planning, cost estimation, logical plan construction
- `execution/` sub-module: physical execution, operator dispatch
- `cache_integration/` sub-module: cache read/write coordination
- Top-level `engine.rs` becomes a thin coordinator (~300 lines)

### server.rs — prism-mcp (11,429 total / ~5,774 prod)

**Step A (S-DECOMP-SERVER-A) — params module:**
Extract tool parameter parsing/validation into `params/` sub-module (~1,500 lines).

**Step B (S-DECOMP-SERVER-B) — per-family dispatch:**
Split per MCP tool family: `tools/query.rs`, `tools/sensor.rs`, `tools/admin.rs`,
`tools/resources.rs`. Top-level `server.rs` becomes router (~400 lines).

### materialization.rs — prism-query (7,083 total / ~4,664 prod)

Candidate boundaries:
- `fan_out.rs`: concurrent sensor dispatch + result collection
- `cache.rs`: materialization cache interaction
- `session.rs`: session-scoped materialization state
- `resolve.rs`: column/table resolution

Top-level `materialization.rs` becomes coordinator (~500 lines).

### error_mapping.rs — prism-mcp (5,855 total / ~2,577 prod)

Candidate boundaries:
- `structured.rs`: structured error content builders (StructuredErrorContent variants)
- `codes.rs`: MCP error code mapping + E-QUERY/E-SENSOR translation

### pipeline.rs — prism-spec-engine (4,578 total / ~1,895 prod)

Candidate boundaries:
- `request_builder.rs`: auth + pagination + header construction
- `extraction.rs`: response body extraction + JSONPath
- `normalization.rs`: column normalization + type coercion

### boot.rs — prism-bin (4,023 total / ~3,000+)

Candidate boundaries:
- `steps.rs`: individual boot step functions (step1 through step12)
- `credential.rs`: credential loading + vault integration

### S-DECOMP-REMAINING files

These six files are lower priority; they decompose after the three highest-impact
splits (engine/server/materialization) reduce the allowlist count:

| File | Crate | Strategy |
|------|-------|----------|
| ast.rs | prism-query | Strongest cohesion; split only if a clear sub-module emerges post-engine split |
| infusion_udf.rs | prism-query | registry.rs + execution.rs |
| spec_driven_adapter.rs | prism-bin | normalization.rs + mapping.rs |
| error.rs | prism-core | sensor_errors.rs + query_errors.rs |
| filter_parser.rs | prism-query | Lowest priority; parser cohesion high |
| prism_describe.rs | prism-mcp | Single tool handler; split only if a second describe variant is added |

---

## Scheduling Note

| Story | Priority | Scheduling Constraint |
|-------|----------|-----------------------|
| S-DECOMP-RATCHET-GATE | P2 | FIRST; post-Wave-A + post-test-soc remediation |
| S-DECOMP-ENGINE-A | P3 | After ratchet gate |
| S-DECOMP-ENGINE-B | P3 | After ENGINE-A |
| S-DECOMP-SERVER-A/B | P3 | After ratchet gate; parallel with ENGINE-B |
| S-DECOMP-MATERIALIZATION | P3 | After ratchet gate |
| S-DECOMP-ERROR-MAPPING | P3 | After ratchet gate |
| S-DECOMP-PIPELINE | P3 | After ratchet gate |
| S-DECOMP-BOOT | P3 | After ratchet gate |
| S-DECOMP-REMAINING | P3 | After the five high-impact splits above |

**Full story materialization is DEFERRED** — product-owner authors story specs only
when the epic is scheduled (post-Wave-A). This document is a planning artifact only;
it does not constitute story authorization.

---

## Tech Debt Anchors

- **TD-DECOMP-RATCHET-001 (P2):** CI gate + `just check-file-sizes` + allowlist
- **TD-DECOMP-EPIC-001 (P3 umbrella):** 12-file decomposition plan (this document)

Both filed in `.factory/tech-debt-register.md` v2.25, D-1899.
