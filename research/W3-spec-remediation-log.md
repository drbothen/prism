---
document_type: remediation-log
level: L4
version: "1.0"
producer: story-writer
timestamp: "2026-05-04T00:00:00Z"
status: complete
scope: "Wave 3 spec remediation — pre-TDD quality gate (D-223)"
inputs:
  - .factory/research/S-3.01/uncertainty-map.md
  - .factory/research/W3-BATCH-uncertainty-summary.md
  - .factory/research/W3-library-versions.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md (v4.32)
  - .factory/stories/S-3.01-prismql-parser.md through S-3.13-dynamic-table-availability.md
---

# Wave 3 Spec Remediation Log

## Overview

Pre-TDD spec remediation burst executed 2026-05-04 per directive D-223 ("fully implement
wave 3") following RED uncertainty scan results from three background agents. This log
documents all changes applied, including per-story changes, cross-story changes, and items
flagged for TDD-time verification rather than spec-time resolution.

---

## Sub-task A: RED Issue Fixes (3 stories)

### S-3.01 — PrismQL Parser (v1.5 → v1.6)

**File Structure table "Create" → "Extend":**
- `crates/prism-query/Cargo.toml` Action changed from "Create" to "Extend" (merge new deps
  alongside existing ones; do NOT recreate — crate already exists with S-2.08 + S-3.2.08 code)
- `crates/prism-query/src/lib.rs` Action changed from "Create" to "Extend" (add new pub mod
  declarations alongside existing crowdstrike_session/materialization/types)

**Chumsky version confirmed "0.12":**
- Library table already had `"0.12"` — confirmed correct per crates.io verification
  (2026-05-04; `chumsky 0.12.0` is the current stable; 0.11.0 was yanked; 1.0.0-alpha.8 is dormant)
- Error type: `Rich<'a, char>` via `extra::Err<Rich<...>>` — NOT `Simple<Token>`

**Kani exact-pinned:**
- `kani-verifier = "=0.67.0"` (dev-dep, exact pin per research recommendation)
- Install procedure noted: `cargo install --locked kani-verifier && cargo kani setup`
- Out-of-band nightly; does not affect workspace MSRV

**VP-015 depth limit reconciled:**
- Task 11 note updated: "e.g. 32" placeholder removed; canonical limit is **64**
  per BC-2.11.006, DI-019, EC-002
- VP-015 file updated separately (see Sub-task: VP-015 edit)

**MSRV note added:** Rust 1.85 (driven by arrow/datafusion/rocksdb/proptest)

### S-3.05 — Pagination and Caching (v1.8 → v1.9)

**lru version conflict resolved — CRITICAL RED fix:**
- Prior story claimed `lru = "0.12.x"` in Library table
- Workspace `crates/prism-query/Cargo.toml` already has `lru = "0.17"` as dev-dep
- 0.12.x is 5 major versions stale; API redesigned multiple times since 0.12
- **Decision:** Use `moka = "0.12"` (features = ["sync"]) for the cross-query response
  cache — chosen for async-safe eviction, built-in TTL support, and production grade
  quality. `lru = "0.17"` retained for any in-process LRU registries that need it.

**Caching Context Summary table added:**
Documents all 3 distinct caching contexts and their library assignments:
- Cross-query response cache: `moka "0.12"` (process-global, TTL-based)
- In-query dedup cache: `stdlib HashMap + Arc` (per-execute() ephemeral — S-3.11 scope)
- Sensor session registries: `lru "0.17"` (in-process LRU)

**Library table updated:**
- `lru 0.12.x` → `moka "0.12"` (features = ["sync"])
- `uuid 1.x` → `uuid { version = "1.23", features = ["v7", "serde"] }`
- `tracing 0.1.x` → `tracing "0.1.44"`
- `kani (dev)` → `kani-verifier "=0.67.0"` (dev-dep, exact pin)

### S-3.07 — Write Execution Pipeline (v1.6 → v1.7)

**DataFusion 53.1 write API uncertainty flagged:**
- `TableProvider::insert_into` signature with `InsertOp` enum: unconfirmed for 53.1
- UPDATE/DELETE trait surface: unconfirmed (may be separate methods or via InsertOp)
- **Resolution:** Story remains at 5 pts; implementer MUST read DataFusion 53.1 source
  before Task 9. If extension-planner work is needed, file W3-FIX-* story rather than
  expanding S-3.07 scope.
- Dev Notes updated with explicit TDD-gate language
- Library table: `datafusion 53.x` → `datafusion "53.1"` with API flag note

**AST module path (see also Sub-task D):**
- Previous Story Intelligence updated: S-3.07 now explicitly states it consumes AST
  from `prism_query::write_ast` (canonical module path established by S-3.06)

---

## Sub-task B: Version Pin Propagation (all 13 stories)

All stories received consistent version pins per W3-library-versions.md research
(versions verified against crates.io API on 2026-05-04):

| Library | Canonical Pin | Stories Updated |
|---------|--------------|-----------------|
| `datafusion` | `"53.1"` | S-3.02, S-3.03, S-3.07, S-3.09, S-3.10, S-3.12, S-3.13 |
| `arrow` | `"58"` (transitive) | S-3.02, S-3.08, S-3.11 |
| `chumsky` | `"0.12"` | S-3.01 (confirmed), S-3.06 |
| `moka` | `"0.12"` (features = ["sync"]) | S-3.05 (new — replaces lru) |
| `kani-verifier` | `"=0.67.0"` (exact, dev-dep) | S-3.01, S-3.04, S-3.05 |
| `proptest` | `"1.11"` | S-3.02, S-3.04 |
| `tracing` | `"0.1.44"` | S-3.02, S-3.04, S-3.08, S-3.09, S-3.10, S-3.12 |
| `serde` | `"1"` (features = ["derive"]) | S-3.03, S-3.04, S-3.09 |
| `regex` | `"1.12"` | S-3.04 |
| `uuid` | `{ version = "1.23", features = ["v7", "serde"] }` | S-3.05 |
| `ariadne` | `"0.4"` | S-3.06 |

**MSRV note (Rust 1.85)** added to all 13 stories: driven by arrow 58 / datafusion 53.1 /
rocksdb 0.24 / proptest 1.11.

---

## Sub-task C: BC Anchor Backfill (6 stories)

All 6 stories had empty `behavioral_contracts: []` and `anchor_bcs: []`. Per VSDD
requirement, each was anchored to the closest active BCs from BC-INDEX v4.32.

These are **proxy anchors** — the story behaviors are genuine osquery-inspired
enhancements without dedicated BCs. Dedicated BC authorship by PO is recommended
before any of these stories transitions to `status: ready`.

| Story | BCs Added (proxy) | Rationale |
|-------|------------------|-----------|
| S-3.08 | BC-2.11.001, BC-2.11.005, BC-2.11.010, BC-2.11.012 | Query tool response schema; materialization injection point; explain output; virtual field co-location |
| S-3.09 | BC-2.11.001, BC-2.08.008 | Query _meta.metrics envelope; get_diagnostics fanout subsystem |
| S-3.10 | BC-2.11.007, BC-2.11.010 | Cost model feeds push-down pipeline; explain cost_estimates output |
| S-3.11 | BC-2.11.005, BC-2.11.011 | Ephemeral cache scope; client_id isolation requirement |
| S-3.12 | BC-2.11.007, BC-2.16.001 | Field-selection push-down; spec schema extension |
| S-3.13 | BC-2.16.007, BC-2.16.001, BC-2.11.001 | Hot reload table delta; spec-driven registration; E-QUERY-001 response |

Each story received:
- `behavioral_contracts:` frontmatter array populated
- `anchor_bcs:` frontmatter array populated
- `anchor_capabilities:` frontmatter populated
- `anchor_subsystem:` frontmatter populated
- `## Behavioral Contracts` table added (body)
- `## Behavioral Contract Linkage` table added (body — maps BCs to ACs)

---

## Sub-task D: Cross-Story AST Coherence (S-3.06 + S-3.07)

**Problem:** Both S-3.06 (write parser) and S-3.07 (write execution) reference shared
write AST types (WriteNode, DmlNode, WriteArg, Assignment) without naming the module path.
The 8-way Tier 3 parallel execution could produce duplicate AST definitions if the
module path is not canonical.

**Fix applied:**

S-3.06 Library & Framework Requirements section now states:
> **AST Module Path:** `prism_query::write_ast` — this is the canonical module path for
> the write AST types (`WriteNode`, `DmlNode`, `WriteArg`, `Assignment`) produced by this
> story. S-3.07 consumes AST from `prism_query::write_ast` per the dependency graph.

S-3.07 Previous Story Intelligence section now states:
> These types are located in `prism_query::write_ast` — the canonical module path
> established by S-3.06 §AST Module Path. Import from there; do NOT define a second copy
> of these AST types in S-3.07 scope.

---

## Sub-task: VP-015 Depth Limit Edit

**File:** `.factory/specs/verification-properties/vp-015-query-nesting-depth.md`
**Change:** v1.4 → v1.5

Property Statement changed from:
> "...depth exceeds the configured ceiling (e.g. 32)..."

To:
> "...depth exceeds the configured ceiling (**64**)..."

With note: "Canonical limit: 64 (per BC-2.11.006, DI-019, EC-002 in S-3.01). The prior
'e.g. 32' was an illustrative placeholder; 64 is the canonical value."

This resolves the three-way inconsistency: VP-015 said 32; BC-2.11.006 + S-3.01 EC-002 +
DI-019 all said 64. Majority of citations + grammar complexity requirement → 64 wins.

---

## Sub-task E: STORY-INDEX Update (v2.04 → v2.05)

- Version bumped: `v2.04` → `v2.05`
- Full Story List BC counts updated for S-3.08 through S-3.13 (0 → N proxy)
- Story version annotations added to all 13 W3 story rows
- BC Traceability Matrix updated: all newly anchored proxy BCs added
- Note updated: clarifies osquery stories now have proxy BC anchors
- Changelog entry v2.05 added with full summary

---

## Items Flagged for TDD-Time Verification (NOT Fixed in Spec)

These items were identified but left for TDD because they require live codebase inspection
or API surface confirmation:

| Item | Story | Flag | Notes |
|------|-------|------|-------|
| `SessionContext::new_with_config_rt` constructor signature | S-3.02 | TDD-gate | DataFusion 53.x constructor name — Dev Notes already has "API has changed" warning; kept as-is |
| `SessionContext::create_logical_plan` vs `state().create_logical_plan(sql)` | S-3.03 | TDD-gate | May live on `.state()` in 53.x — kept existing warning |
| RocksDB-vs-toml alias persistence inconsistency | S-3.04 | TDD-gate | Internal contradiction at body line 412; S-3.04 Architecture Compliance Rules say `aliases.toml` (file-based); this is authoritative. Implementer should disregard any RocksDB CF mention for alias storage |
| `MemoryPool::reserved()` API name | S-3.09 | TDD-gate | May be `consumed()` or `current()` in DataFusion 53.1; TDD-gate note added to Library table |
| DataFusion `insert_into` + `InsertOp` + UPDATE/DELETE trait surface | S-3.07 | TDD-gate (RED) | Explicitly flagged in Dev Notes; story sized assuming extension hooks exist |
| DataFusion JOIN order hint behavior (manual ordering vs optimizer override) | S-3.10 | TDD-gate | If optimizer overrides Prism ordering, document and file follow-on |
| DataFusion `OptimizerRule` trait shape (`try_optimize` vs `rewrite()`) | S-3.12 | TDD-gate | AST walker approach preferred; DataFusion optimizer path is secondary |
| DataFusion `register_table`/`deregister_table` concurrency semantics | S-3.13 | TDD-gate | CatalogProvider fallback approach noted in Library table |
| `datafusion-sql::DmlStatement` public export in 53.x | S-3.06 | TDD-gate | Optional reference; Dev Notes already says "verify this pattern" |
| TenantId == org_id canonicality (ADR-008 re-keying) | S-3.11 | Flag | S-3.1.01 OrgId migration may rename TenantId; CacheKey struct may need update |
| Single DataFusion 53-api-confirmed memo | Cross-cutting | Research | Batch uncertainty summary recommended a `datafusion-53-api-confirmed.md` memo for all 8 API surfaces. This was NOT produced in this burst — remains a recommended pre-kickoff research task for the Wave 3 implementer team |

---

## Files Modified

| File | Old Version | New Version | Change Type |
|------|-------------|-------------|-------------|
| `.factory/stories/S-3.01-prismql-parser.md` | 1.5 | 1.6 | RED fix (File Structure Create→Extend, Kani pin, VP-015 note) |
| `.factory/stories/S-3.02-query-materialization.md` | 1.7 | 1.8 | Version pins |
| `.factory/stories/S-3.03-explain-query.md` | 1.4 | 1.5 | Version pins |
| `.factory/stories/S-3.04-alias-system.md` | 1.7 | 1.8 | Version pins (kani exact, regex 1.12, proptest 1.11) |
| `.factory/stories/S-3.05-pagination-caching.md` | 1.8 | 1.9 | RED fix (lru→moka, caching context table) |
| `.factory/stories/S-3.06-prismql-write-parser.md` | 1.5 | 1.6 | Version pins, AST module path canonical name |
| `.factory/stories/S-3.07-write-execution.md` | 1.6 | 1.7 | RED flag (DataFusion write API TDD-gate), AST consumption anchor |
| `.factory/stories/S-3.08-hidden-columns.md` | 1.4 | 1.5 | BC anchor backfill + version pins |
| `.factory/stories/S-3.09-query-profiling.md` | 1.4 | 1.5 | BC anchor backfill + version pins |
| `.factory/stories/S-3.10-cost-estimation.md` | 1.4 | 1.5 | BC anchor backfill + version pins |
| `.factory/stories/S-3.11-in-query-caching.md` | 1.4 | 1.5 | BC anchor backfill + scope boundary note |
| `.factory/stories/S-3.12-column-pruning.md` | 1.4 | 1.5 | BC anchor backfill + OptimizerRule TDD-gate |
| `.factory/stories/S-3.13-dynamic-table-availability.md` | 1.6 | 1.7 | BC anchor backfill + register_table TDD-gate |
| `.factory/stories/STORY-INDEX.md` | v2.04 | v2.05 | Story version cells, BC proxy counts, traceability matrix |
| `.factory/specs/verification-properties/vp-015-query-nesting-depth.md` | 1.4 | 1.5 | Depth limit 32→64 |

**Files NOT modified (non-goals):**
- BC files (only read BC-INDEX; did not modify any BC-2.*.*.md file)
- ADR files
- PRD or ARCH-INDEX
- HS files
- wave-state.yaml or STATE.md
- VP files other than vp-015 (single permitted edit)
