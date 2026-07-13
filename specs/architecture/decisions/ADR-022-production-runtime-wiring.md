---
document_type: adr
adr_id: "ADR-022"
title: "Production Runtime Wiring — prism-bin Chassis, Boot Sequence, Wiring Contracts, Infusion Fate, Hot-Reload Watcher, MCP Topology"
status: ACCEPTED
date: "2026-05-17"
version: "1.17"
producer: architect
subsystems_affected: [SS-06, SS-10, SS-11, SS-16, SS-17, SS-19]
supersedes: null
superseded_by: null
inputs:
  - .factory/cycles/wave-4-operations/workspace-audit-2026-05-08.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/architecture/decisions/ADR-021-bc-vp-promotion-lifecycle.md
  - .factory/specs/architecture/decisions/ADR-020-story-status-taxonomy-reform.md
  - .factory/specs/architecture/module-decomposition.md
  - .factory/policies.yaml
  - .factory/STATE.md
anchor_stories: []
references_phase3_siblings: [ADR-005, ADR-020, ADR-021]
locked_decisions: []
runtime_deliverables:
  - crates/prism-bin/src/main.rs           # binary entry point
  - crates/prism-bin/src/boot.rs           # boot sequence orchestrator
  - crates/prism-bin/src/cli.rs            # clap CLI surface
  - crates/prism-bin/src/signals.rs        # signal handlers (SIGTERM, SIGHUP)
  - crates/prism-mcp/src/server.rs         # rmcp 1.7 PrismServer struct
  - crates/prism-mcp/src/tools/mod.rs      # tool router + all 35+ tool implementations
  - crates/prism-spec-engine/src/hot_reload.rs  # HotReloadWatcher::start/stop (notify 7)
wiring_deferred_to: null  # This ADR IS the wiring specification — no further deferral
input-hash: "1a916c8"
---

# ADR-022: Production Runtime Wiring — prism-bin Chassis, Boot Sequence, Wiring Contracts, Infusion Fate, Hot-Reload Watcher, MCP Topology

## Status

ACCEPTED 2026-05-08, v1.0. Effective immediately; implementation is tracked by the six story
seeds in §G. This ADR satisfies POL-15 (`runtime_wiring_required_for_accepted_adrs`) by
providing the authoritative wiring specification for: AD-005 (rmcp MCP server), AD-007
(arc-swap hot reload), AD-018 (notify filesystem watcher), AD-022 (write operations), and
the binary entry-point for the entire Prism runtime.

---

## Context

The 2026-05-08 workspace audit (D-301, F-AUD-D2-13, F-AUD-D1-01..06) identified one
structural gap underlying six P0 findings: **no binary in the Prism workspace constructs
ConfigManager, QueryEngine, WriteExecutor, sensor registries, or an MCP server.** The
runtime is functionally absent:

- `prism-mcp/src/lib.rs` is a 10-line stub (verified: `wc -l` = 10; only `pub mod
  safety_envelope` and `pub mod tool_registry` — no rmcp server, no tool router, no binary).
- `QueryEngine::execute` in `engine.rs` was `todo!("S-3.02 — QueryEngine::execute")` until
  resolved by S-3.02-FOLLOWUP-RUNTIME (PR #141 c6dd6602, 2026-05-10). HISTORICAL.
- `run_materialization_pipeline` in `materialization.rs` was `todo!("S-3.02 — ...")` until
  resolved by S-3.02-FOLLOWUP-RUNTIME. HISTORICAL.
- `RocksDbTableProvider::schema/scan/register_internal_tables` in `internal_tables.rs`
  were `todo!("S-3.02 — ...")` until resolved by S-3.02-FOLLOWUP-RUNTIME. HISTORICAL.
- `WriteExecutor::execute` Phase 3 fetch at `write_pipeline.rs` is `let fetched_records:
  Vec<...> = vec![];` — hardcoded empty, never fetches records.
- `WriteCapableTableProvider::insert_into/delete_from/update` at
  `write_table_registration.rs` return `DataFusionError::NotImplemented("S-3.07-pending")`.
- `SensorAdapter::write()` default at `adapter.rs` returns `WriteNotImplemented` for all
  four built-in sensors; no concrete override exists.
- `HotReloadWatcher::start/stop` at `hot_reload.rs` are
  `unimplemented!("S-1.12: ... Red Gate stub")`.
- `ConfigManager::new` and `parse_spec_directory` are called only from test code; no production
  binary instantiates them (verified: grep for callers outside test files returns zero matches).

Bundles A (taxonomy reform), B (runtime gap), D (doc cleanup) are the three cleanup epics
approved at D-302. This ADR is the Phase B-0 architecture output for Bundle B.

---

## Decision

The Prism workspace acquires a `prism-bin` binary crate as the sole `[[bin]]` target. The
binary implements an ordered 11-step boot sequence (§B) that constructs all subsystems in
dependency order and exposes them via an rmcp 1.7 stdio MCP server. Infusions (§D) are
retained but deferred to Wave 5 (REDO). Hot-reload watcher (§E) runs as a background task
post-boot. Full wiring contracts per subsystem are specified in §C.

---

## Rationale

The workspace audit (D-301) confirmed that no binary constructs or wires any subsystem. This
ADR closes that gap by specifying the wiring contracts, boot sequence, and story seeds for
implementation. See §A–§G for per-subsystem rationale. The single-binary decision is driven
by the per-analyst stdio deployment model (AD-005). The infusion REDO decision is driven by
critical-path analysis (§D — MCP server and query engine are blocking; infusions are not).

---

## Consequences

- `prism-bin` crate added to workspace at `S-WAVE5-PREP-01` — all subsystems become wired.
- `QueryEngine::execute` gap closed by S-3.02-FOLLOWUP-RUNTIME (merged PR #141 c6dd6602).
- Hot-reload watcher remains `unimplemented!()` until S-1.12-FOLLOWUP.
- MCP server (`PrismServer`) absent from `prism-mcp` until S-5.01-FOLLOWUP-MCP-BOOT.
- Write path gaps (Phase 3 fetch, SQL DML) remain until W3-FIX-S307-001/002/003.
- S-1.14 reclassified `staged-redo`; infusion framework retained at Wave 5 scope.

---

## §A — prism-bin Crate Chassis

### Crate identity

`prism-bin` is a new binary crate at `crates/prism-bin/`. It is the **only** `[[bin]]` target
in the Prism workspace. All other crates remain libraries. The crate is already declared in
`module-decomposition.md` as `(planned for future waves)` and listed in the ARCH-INDEX layered
diagram as Layer 4. This ADR graduates it from planned to specified.

The crate is NOT added to `Cargo.toml` workspace `members` until `S-WAVE5-PREP-01` ships it;
the workspace currently has 24 members and no binary target at all.

### CLI surface (clap)

Arg-parsing: `clap` (version must match workspace convention — add to `Cargo.toml`
`[workspace.dependencies]` at the version currently latest stable, ~4.x). The workspace has
no existing clap pin; implementer sets the pin.

**Subcommands (minimum viable set):**

| Subcommand | Purpose | Exit codes |
|---|---|---|
| `start` | Boot and run (blocks until SIGTERM/Ctrl-C) | 0 clean, 1 generic, 2 config-invalid, 3 sensor-fail, 4 internal-error |
| `query <query-string>` | Execute one PrismQL query; output JSON to stdout; exit | 0 result, 1 parse-error, 2 config-invalid, 3 sensor-fail |
| `validate-config` | Parse config + sensor TOMLs; report validity; exit | 0 valid, 2 config-invalid |
| `version` | Print semantic version + build metadata; exit 0 | 0 always |

**Deferred subcommands (post-MVP):** `migrate`, `debug-sensor`, `shell`. These are NOT
specified in this ADR and MUST NOT block `S-WAVE5-PREP-01`.

### Exit-code contract (canonical)

```
0  — success / clean shutdown
1  — unhandled error (generic; includes unexpected panics caught by panic hook)
2  — config-invalid (TOML parse error, schema validation failure, credential ref resolution failure)
3  — sensor-fail (a required sensor adapter failed to initialize at boot; non-required adapters degraded-ok)
4  — internal-error (runtime invariant violation; query engine init failed; RocksDB open failed)
5  — permission-denied (credential store access denied at boot)
```

These exit codes are the contract surface between `prism-bin` and any shell wrapper or
integration test. They must be documented in the binary's `--help` output and in `installation.md`.

### Logging / tracing initialization

`tracing` crate (already in workspace, e.g., `prism-query/Cargo.toml:tracing = "0.1"`).
Subscriber: `tracing-subscriber` with `EnvFilter` (allows `RUST_LOG` override). Default level:
`info`. JSON format for machine-readable log lines (configurable via `PRISM_LOG_FORMAT=json|pretty`).
Initialization occurs before any other boot step — the first log line should be the Prism
version string (for audit trail of deployments).

Panic hook: register a custom `std::panic::set_hook` that emits a `tracing::error!` log
before unwinding. This ensures panics appear in structured logs (not just stderr) and exit
code 1 is returned by the process via `std::process::exit(1)` in the hook.

### MCP transport (stdio default)

Per the per-analyst deployment model (CLAUDE.md, memory), the MCP server transport is **stdio**
(stdin reads MCP JSON-RPC requests; stdout writes MCP JSON-RPC responses). This matches how
Claude Code / MCP clients connect. No TCP or Unix socket transport is specified for MVP.

The `start` subcommand connects the rmcp 1.7 server to stdio transport. This is the only
transport mode in scope for `S-WAVE5-PREP-01` and `S-5.01-FOLLOWUP-MCP-BOOT`.

---

## §B — Boot Sequence Specification

The boot sequence for `prism start` is ordered and idempotent. Each step either completes
synchronously before the next step begins, or spawns a background task and registers it in
the task tracker. **No MCP traffic is accepted until step 8 (MCP server start).**

```
Step 1   [BLOCKING] Tracing init
         Action: initialize tracing subscriber (EnvFilter + JSON/pretty format per PRISM_LOG_FORMAT)
         Failure: log to stderr, exit 4

Step 2   [BLOCKING] Config load
         Action: read prism.toml + aliases.toml from config dir
           ($PRISM_CONFIG_DIR if set, else `dirs::config_dir().join("prism")`:
           ~/.config/prism/ on Linux ($XDG_CONFIG_HOME-aware),
           ~/Library/Application Support/prism/ on macOS,
           %APPDATA%\prism\ on Windows)
         Action: validate schema (config-schema.md contract)
         Failure: exit 2 (config-invalid)

Step 3   [BLOCKING] OrgRegistry init
         Action: construct OrgRegistry from config (org_id + org_slug pairs per ADR-006)
         Failure: exit 2 (org identity config invalid)

Step 4   [BLOCKING] Sensor TOML spec load
         Action: call parse_spec_directory(config.spec_dir) → ConfigSnapshot
         Action: construct ConfigManager::new(snapshot) wrapped in Arc<ArcSwap<ConfigManager>>
         Action: validate all sensor specs (format + credential ref resolution)
         Failure: exit 2 (sensor spec parse failure)
         NOTE: currently parse_spec_directory (prism-spec-engine/src/config_manager.rs)
               and ConfigManager::new are real but called only from tests. This step is the
               first production call site.

Step 5   [BLOCKING] Credential store init
         Action: initialize CredentialStore (keyring or AES-file backend per prism.toml)
         Action: resolve all credential refs declared in sensor specs (verify access; values NOT
                 loaded into memory — reference-based model per AD-017)
         Failure: exit 5 (permission-denied) or exit 2 (config-invalid ref)

Step 6   [BLOCKING] Audit subsystem init
         Action: construct AuditEmitter (prism-audit); open audit buffer (RocksDB CF: audit_buffer)
         Failure: exit 4 (internal-error — audit is required for SOC 2)

Step 7   [BLOCKING] Storage + internal-tables provider init
         Action: open RocksDB with all 19 column families (per AD-004; prism-storage opens
                 the full StorageDomain::all() set — prism-core storage.rs ALL_DOMAINS,
                 16 S-1.01 domains + 3 S-1.02 domains: credentials, feature_flags, scheduler)
         Action: call register_internal_tables (prism-query/src/internal_tables.rs)
                 — was todo!("S-3.02 — register_internal_tables") until S-3.02-FOLLOWUP-RUNTIME
         Action: construct AdapterRegistry::new() — registry starts EMPTY; spec-driven adapter
                 population happens at step 9A (inside step9_start_mcp_server, before QueryEngine
                 construction) via step9a_populate_adapter_registry
         Failure: exit 4 (RocksDB open failure or internal-tables registration failure)

Step 7.5 [BLOCKING] Plugin runtime load  ← see ADR-023 §C4 (authoritative placement spec)
         ADR-023 §C4 supersedes this ADR for plugin-load step placement (Source-of-Truth
         Precedence Rule 2: later, more-specific ADR wins for the surface it owns).
         Action: call PluginRuntime::load_all_plugins (crates/prism-spec-engine/src/plugin/mod.rs)
         Action: emit tracing::warn! + stderr banner — plugin signing not yet implemented
                 (TD-PLUGIN-SIGNING-001); PRISM_DISABLE_PLUGIN_LOAD=1 skips this step entirely
         Action: emit audit log entry per loaded plugin: event_type: plugin_load_unsigned
         Failure: exit 4 (plugin load failure)
         NOTE: The fractional step number (7.5) is intentional — it avoids a cascading
               renumber of this canonical step table, boot.rs function names
               (e.g. step9_start_mcp_server), and historical STATE.md narrative.
               Authoritative behavior: BC-2.22.001 §Sequencing Invariant,
               §Pre-Traffic Gate Invariant condition 6, §Postconditions, §Exit-Code Map.
         Delivered by: PLUGIN-PREREQ-D (depends on PLUGIN-PREREQ-F for PluginRuntime infra)

Step 7.5b [BLOCKING] PluginAuthProvider construction
         Action: call validate_and_construct_auth_providers(snapshot, &plugin_result.runtime)
                 — validates every sensor spec that declares auth_plugin; constructs one
                 Arc<PluginAuthProvider> per matching plugin ID; stores in
                 plugin_result.plugin_auth_providers
         Action: plugin_auth_providers is threaded through run_boot_sequence into
                 step9_start_mcp_server for use at step 9A
         Failure: exit 4 (plugin auth provider construction failure)
         Delivered by: PLUGIN-MIGRATION-001-E

Step 7.5c [BLOCKING] Dynamic write-tool registration
         Action: register built-in write tools into DYNAMIC_WRITE_TOOLS before step 8
         Action: must execute AFTER step 7.5 plugin-load AND BEFORE step 8
                 mark_query_phase_started()
         Failure: exit 4

Step 8   [BLOCKING → BACKGROUND] QueryEngine + WriteExecutor construction
         First statement: prism_query::invalidation::mark_query_phase_started() — closes the
               write-registration window before QueryEngine::new(). See ADR-026 §D7 v1.23.
         Action: construct QueryEngine (prism-query); bind AdapterRegistry + StorageBackend
         Action: construct WriteExecutor (prism-query); bind feature-flag check + capability check
         Note: QueryEngine::execute in engine.rs was todo!() — resolved by S-3.02-FOLLOWUP-RUNTIME
               (PR #141 c6dd6602, 2026-05-10)
         After construction completes: engine accepts queries (via MCP tools)
         Failure: exit 4

Step 9   [BACKGROUND] MCP server start
         Step 9A [within step 9, before QueryEngine construction]:
           Action: call step9a_populate_adapter_registry(&resolved_spec_map, &org_registry,
                   &plugin_auth_providers, &mut adapter_registry) — populates AdapterRegistry
                   with one SpecDrivenSensorAdapter per (OrgId, SensorId) pair from the
                   resolved spec map (S-DEMO-001 / BC-2.22.001 §Step 9A)
           Action: emit boot.step9a.adapter_registry_populated event with sensor_count +
                   org_count fields (BC-2.16.002 catalog row)
           Failure: exit 4
         After step 9A: AdapterRegistry is fully populated; QueryEngine is constructed and
           bound to the populated registry
         Action: call PrismServer::new(engine, write_executor, audit_emitter, security_config)
         Action: bind rmcp 1.7 stdio transport
         Action: register all tools via #[tool_router] macro (§F tool inventory)
         Action: enable prompt-injection defense middleware (§F, BC-2.09.001..008)
         Once: write "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}" to stdout
         MCP server now accepting tool calls
         Failure: log error + exit 4

Step 10  [BACKGROUND] Hot-reload watcher install
         Action: call HotReloadWatcher::start(manager.clone(), config.spec_dir, debounce_ms=500)
         Action: currently unimplemented!() at hot_reload.rs — S-1.12-FOLLOWUP resolves
         Background task: fs events → validate → arc-swap (§E)
         Failure: log warning (non-fatal — degrade gracefully without reload; alert is emitted)

Step 11  [BACKGROUND] Signal handler install
         Action: register tokio signal handlers for SIGTERM + SIGHUP
         SIGTERM: initiate graceful shutdown (drain in-flight queries → close MCP server →
                  flush audit buffer → close RocksDB → exit 0)
         SIGHUP: trigger manual config reload (same code path as hot-reload watcher)
         Failure: log error, continue (OS may still deliver signals)
```

**Traffic gate:** steps 1–8 (inclusive of step 7.5) are blocking. Queries cannot reach
`QueryEngine::execute` until step 8 completes. The MCP server (step 9) only starts after step
8. Steps 10–11 are background and non-fatal; their failure degrades capability but does not
prevent serving queries.

**Idempotency:** If any step fails and the process exits, re-executing `prism start` with
corrected config must successfully complete all steps. No step leaves permanent state corruption
on failure (RocksDB open is the one exception — a crash-incomplete write may require RocksDB
repair, which is a documented operational procedure in `installation.md`).

---

## §C — Wiring Contracts

For each subsystem, the constructor / init / shutdown function that `prism-bin` calls:

### Config Manager (SS-06 / prism-spec-engine)

```rust
// Boot step 4 call site
use prism_spec_engine::config_manager::{parse_spec_directory, ConfigManager};
use arc_swap::ArcSwap;

// File: crates/prism-spec-engine/src/config_manager.rs — REAL (not stubbed)
pub fn parse_spec_directory(spec_dir: &Path) -> Result<ConfigSnapshot, SpecEngineError>;

// File: crates/prism-spec-engine/src/config_manager.rs — REAL (not stubbed)
impl ConfigManager {
    pub fn new(snapshot: ConfigSnapshot) -> Self;
    pub fn current(&self) -> Arc<ConfigSnapshot>;
    pub fn update(&self, new_snapshot: ConfigSnapshot);
}

// prism-bin constructs:
let snapshot = parse_spec_directory(&config.spec_dir)?;
let manager = Arc::new(ArcSwap::from_pointee(ConfigManager::new(snapshot)));
```

Contract gap: none — ConfigManager and parse_spec_directory are real. The gap is the **call
site** (no binary calls them today).

### QueryEngine (SS-11 / prism-query)

```rust
// Boot step 9A + step 8 call sites
use prism_query::engine::QueryEngine;

// File: crates/prism-query/src/engine.rs — constructor is REAL
impl QueryEngine {
    pub fn new(registry: Arc<AdapterRegistry>, storage: Arc<dyn RocksStorageBackend>,
               ocsf: Arc<OcsfNormalizer>) -> Self;
    // execute and execute_scheduled: resolved by S-3.02-FOLLOWUP-RUNTIME (PR #141 c6dd6602)
    pub async fn execute(&self, query_str: &str, options: QueryOptions)
        -> Result<QueryResult, PrismError>;
}
```

Contract note: AdapterRegistry is populated by step9a_populate_adapter_registry (called from
within step9_start_mcp_server, before QueryEngine::new()) so that QueryEngine is bound to a
fully-populated registry from construction (S-DEMO-001). The AdapterRegistry::new() constructed
at step 7 starts empty; step 9A fills it with one SpecDrivenSensorAdapter per (OrgId, SensorId).
Historical gaps: `QueryEngine::execute` and `QueryEngine::execute_scheduled` in engine.rs were
`todo!()`. Also: `run_materialization_pipeline` and `resolve_source_refs` in materialization.rs,
and `RocksDbTableProvider::{schema,scan}`, `register_internal_tables` in internal_tables.rs were
`todo!()`. All resolved by `S-3.02-FOLLOWUP-RUNTIME` (PR #141 c6dd6602, 2026-05-10). HISTORICAL.

### WriteExecutor (SS-11 / prism-query)

```rust
// Boot step 8 call site
use prism_query::write_pipeline::WriteExecutor;

// Constructor: REAL (not stubbed)
impl WriteExecutor {
    pub fn new(feature_flags: Arc<FeatureFlagStore>, audit: Arc<AuditEmitter>) -> Self;
    // execute has structural gap — Phase 3 fetch hardcoded empty:
    pub async fn execute(&self, plan: WritePlan)
        -> Result<WriteExecutionReport, PrismError>;  // write_pipeline.rs — empty vec![]
}
```

Contract gaps:
- `write_pipeline.rs` — Phase 3 fetch returns `vec![]` (never fetches records).
- `adapter.rs` — `SensorAdapter::write()` default returns `WriteNotImplemented`; no
  concrete override exists for CrowdStrike, Cyberint, Claroty, or Armis.
- `write_table_registration.rs` — `insert_into/delete_from/update` return
  `DataFusionError::NotImplemented("S-3.07-pending")`.

Resolved by: `W3-FIX-S307-001` (concrete adapter write overrides), `W3-FIX-S307-002`
(QueryMaterializer integration into Phase 3 fetch), `W3-FIX-S307-003` (SQL DML routing).

### Hot-Reload Watcher (SS-16 / prism-spec-engine)

```rust
// Boot step 10 call site
use prism_spec_engine::hot_reload::HotReloadWatcher;

impl HotReloadWatcher {
    pub fn new() -> Self;
    // Both are unimplemented!() — contract gap:
    pub fn start(&self, manager: Arc<ConfigManager>, spec_dir: PathBuf,
                 debounce_ms: u64) -> Result<(), SpecEngineError>;  // hot_reload.rs
    pub fn stop(&self) -> Result<(), SpecEngineError>;               // hot_reload.rs
}
```

Contract gap: both methods are `unimplemented!()` at `hot_reload.rs`. Resolved by
`S-1.12-FOLLOWUP`.

### MCP Server (SS-10 / prism-mcp)

```rust
// Boot step 9 call site
// PrismServer does not yet exist — entire rmcp integration is absent from prism-mcp
// Current lib.rs is 10 lines: pub mod safety_envelope + pub mod tool_registry only.

// Contract that S-5.01-FOLLOWUP-MCP-BOOT must implement:
use rmcp::{ServerHandler, tool_router};
use prism_mcp::server::PrismServer;

impl PrismServer {
    pub fn new(engine: Arc<QueryEngine>, write_executor: Arc<WriteExecutor>,
               audit: Arc<AuditEmitter>, security: Arc<SecurityConfig>) -> Self;
    pub async fn serve_stdio(self) -> Result<(), McpError>;
}
```

Contract gap: `PrismServer` struct does not exist. `prism-mcp` has no rmcp dependency (verified:
`prism-mcp/Cargo.toml` has no rmcp pin). Resolved by `S-5.01-FOLLOWUP-MCP-BOOT`.

---

## §D — Infusion Fate Decision

### Decision: S-1.14-REDO at Wave 5

Infusions (SS-19) are **retained in the MVP scope** but the REDO story targets Wave 5, not
Wave 4. S-1.14 is reclassified `status: partial-merge → staged-redo` per ADR-020 taxonomy.

### Rationale

**(a) Critical-path analysis.** Infusions are not blocking any P0 runtime gap that affects
analyst utility. The runtime gap blocking MVP is: no MCP server, no query execution, no boot
binary. Infusions are a data-enrichment layer on top of a working query engine. S-3.02
(QueryEngine), S-WAVE5-PREP-01 (prism-bin), and S-5.01 (MCP server) must ship first —
these are the actual critical path.

**(b) Sensor differentiation value.** The infusion framework (GeoIP enrichment, threat intel
lookup, asset-inventory join) provides significant differentiation for security analysts.
The `| enrich geoip` syntax and TOML-spec-driven enrichment sources are the kind of feature
that MSSPs pay for. Retiring infusions entirely would require removing BCs from the PRD,
retiring VP-049, and reworking the `| enrich` pipe stage syntax — high coordination cost
for no technical benefit.

**(c) Implementation cost.** The infusion framework has substantial structure already:
`InfusionLoader`, `InfusionLruCache`, `MmdbSource`, `CsvSource`, `JsonLookupSource`, and
`plugin_bridge` are all scaffolded with `unimplemented!()` bodies. The `wasmtime 44` dep
is already in `prism-spec-engine/Cargo.toml`. The Kani proof harness for VP-040 exists
(blocked on wasmtime Linker enumeration API — this is a real upstream blocker, not a
design issue). The work is implementer effort, not architectural redesign.

**(d) vs. RETIRE.** Retiring infusions would require:
- Retiring 3+ BCs (BC-2.19.001, BC-2.19.002, BC-2.19.005) per ADR-021 lifecycle.
- Removing `| enrich` syntax from the PRD.
- Retiring VP-049 (proptest for dedup) and VP-040 (Kani).
- Stripping `infusion/` module from `prism-spec-engine`.
- Removing the 3 `.infusion.toml` fixtures.
This is more work than the REDO story and produces a worse product. RETIRE is rejected.

### S-1.14-REDO scope constraints

S-1.14-REDO must implement (in dependency order):
1. `InfusionLoader::{parse,load_all,validate_credentials}`
2. `InfusionLruCache::{get,insert}` — LRU backed by RocksDB CF `infusion_cache`
3. `MmdbSource::{load,enrich_single,enrich_batch}`
4. `CsvSource` and `JsonLookupSource` equivalents
5. `plugin_bridge::enrich_via_plugin` — calls S-1.15 WASM runtime
6. DataFusion UDF registration for `enrich(source, field)` expression
7. Pipe stage `| enrich <source>` compilation to UDF invocation

VP-040 (Kani proof of plugin_bridge correctness) remains blocked until wasmtime provides a
stable Linker enumeration API. VP-040 status stays `harness-only` until the upstream unblocks.
This is documented but does not block the REDO story.

---

## §E — Hot-Reload Watcher Scope

### Decision (AD-018 implementation contract)

The hot-reload watcher runs `notify` crate v7 (already in `prism-spec-engine/Cargo.toml:
notify = "7"`). The watcher specification for `S-1.12-FOLLOWUP`:

### notify-rs integration

```rust
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::sync::mpsc;

// Create the watcher with the recommended platform backend.
// On macOS: FSEvents (accurate, low-latency).
// On Linux: inotify (accurate).
// On Windows: ReadDirectoryChangesW (not in scope — stdio MCP is analyst-workstation only;
//             macOS + Linux are the only supported platforms for MVP).
let (tx, rx) = mpsc::channel();
let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
watcher.watch(spec_dir, RecursiveMode::Recursive)?;
```

### Debounce window

500ms (per AD-018). Canonical implementation: accumulate events into a `HashMap<PathBuf,
EventKind>` keyed on path; flush after 500ms of inactivity. The debounce collapses rapid
file-editor save sequences (multiple writes → one reload trigger). Implementation MUST use
a `tokio::time::interval` or `sleep` loop in a background task — not a busy poll.

### Validation gate (before swap)

Before swapping the ConfigManager snapshot, re-run `parse_spec_directory` in dry-run mode
against the changed directory. If `parse_spec_directory` returns `Err`, the swap is aborted:
- Log `tracing::warn!("hot-reload: validation failed for {path}: {err}; retaining current config")`
- Emit audit entry: `ReloadEvent { kind: ReloadFailed, path, error_code: err.code() }`
- Retain the current arc-swap snapshot unchanged

If `parse_spec_directory` returns `Ok(new_snapshot)`:
- Call `manager.store(Arc::new(ConfigManager::new(new_snapshot)))`
  (arc-swap atomic store — lock-free; in-flight queries using the old snapshot continue safely
  per AD-007)
- Emit audit entry: `ReloadEvent { kind: ReloadSucceeded, path, sensor_count }`
- Log `tracing::info!("hot-reload: config swapped — {} sensors active", n)`

### SIGHUP integration

SIGHUP triggers the same code path as the filesystem watcher:
- Call `try_reload(manager.clone(), spec_dir.clone())` from the signal handler task.
- The reload path is idempotent — SIGHUP during a filesystem-triggered reload is safe
  (both end up calling `parse_spec_directory` + conditional `manager.store`).

### Cross-platform quirks to handle

| Quirk | Handle how |
|---|---|
| macOS FSEvents may batch multiple events into one callback | Debounce window collapses batches |
| inotify may emit `MODIFY` twice for atomic-write editors (write temp + rename) | Filter: only react to `EventKind::Modify` + `EventKind::Create`; ignore `CLOSE_WRITE` if no content change |
| File deletions (sensor TOML removed) | Treat as validation failure — emit `ReloadFailed` audit; do not remove the sensor from the running config (ops safety: accidental deletion should not silently disconnect a sensor) |
| Recursive watch on symlinks | `RecursiveMode::NonRecursive` on spec_dir contents; do NOT follow symlinks — security boundary |
| Watcher task panic | Catch via `tokio::task::spawn` join handle; log + emit degraded-mode alert; hot-reload degrades gracefully (boot-time config remains active) |

---

## §F — MCP Runtime Topology

### rmcp 1.7 integration

`prism-mcp` gains a dependency on `rmcp` version 1.7 (per AD-005; version updated from 1.4 per OQ-1 / F-PASS9-MED-1). This must be added to
`crates/prism-mcp/Cargo.toml` and to the workspace `[workspace.dependencies]` table.

```rust
// Canonical MCP server struct (crates/prism-mcp/src/server.rs — does not yet exist)
use rmcp::{ServerHandler, tool_router, McpServer};

pub struct PrismServer {
    engine: Arc<QueryEngine>,
    write_executor: Arc<WriteExecutor>,
    audit: Arc<AuditEmitter>,
    security: Arc<SecurityConfig>,
    tool_router: ToolRouter<PrismServer>,
}

impl PrismServer {
    pub fn new(...) -> Self { ... }
    pub async fn serve_stdio(self) -> Result<()> {
        let service = McpServer::new(self);
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
        service.serve(stdio_transport(stdin, stdout)).await
    }
}
```

### Tool registration via `#[tool_router]` macro

All tools are registered in a single `#[tool_router]` impl block per the rmcp 1.7 API.
The 35-tool claim in `module-decomposition.md` is aspirational but grounded in BC-2.13.*
(the tool catalog BC). Tool count at MVP target: the full BC-2.13.* catalog.

### Per-tool input validation

Each tool handler MUST:
1. Deserialize the MCP tool parameters from JSON.
2. Validate required fields (non-null, type correct, within bounds).
3. On validation failure: return a structured MCP error response (`code: -32602`,
   message describing the invalid field). NEVER `panic!()` or `unwrap()`.

### Error-code mapping

| PrismError variant | MCP error code | message | suggestion |
|---|---|---|---|
| `ParseError` | -32602 (Invalid params) | "PrismQL parse error: {detail}" | — |
| `SensorError::WriteNotImplemented` | -32003 (Custom) | "Write not supported for sensor: {sensor}" | — |
| `PrismError::PermissionDenied` | -32002 (Custom) | "Feature flag denied: {flag}" | — |
| `PrismError::Timeout` | -32001 (Custom) | "Query timeout exceeded" | — |
| `PrismError::InternalError` | -32000 (Server-defined) | "Internal error" | "See audit log for details." |

> **Note — InternalError code choice:** `-32000` is the `codes::INTERNAL_ERROR` constant in
> `prism-mcp/src/error_mapping.rs` (server-defined range per JSON-RPC spec §5.1). The
> server-defined range (`-32099`..`-32000`) is used for application-level errors because
> `-32603` is the transport-level "Internal error" code reserved by the rmcp layer itself;
> using `-32603` for application errors would conflate transport failures with query-engine
> failures. This behavior has been consistent since PR #163.

### Prompt-injection defense wiring (MANDATORY)

BC-2.09.001..008 are already implemented in `prism-security::injection_scanner`
(S-1.10, status: merged). They MUST be invoked at **every tool entry boundary**:

```rust
// In each tool handler, before any domain logic:
let scan_result = injection_scanner.scan_all(&tool_params.raw_inputs())?;
if scan_result.has_violations() {
    return Err(McpError::custom(-32002, "Input rejected: prompt injection detected"));
}
// Proceed only after clean scan
```

The `safety_envelope` module in `prism-mcp` (already exists) provides the
`ResponseEnvelope` wrapper. Every tool response MUST be wrapped in `ResponseEnvelope::new(result)`
before returning to the MCP caller. This is the existing provenance-framing from S-1.10.

### Tool inventory contract (BC-2.13.* anchors)

The 35-tool architecture claim is bounded by BC-2.13.* behavioral contracts. At MVP, the
tool router MUST implement the tools anchored in BC-2.13.* — specifically the tool categories:
query execution, sensor health, config management, write operations, observability/diagnostics.
The exact tool signatures are specified in `api-surface.md` (v1.6). The implementer
(`S-5.01-FOLLOWUP-MCP-BOOT`) must read `api-surface.md` for the canonical tool inventory
before implementing the router; this ADR does not reproduce the full tool list.

**Non-negotiable:** BC-2.09.001..008 injection defense runs at EVERY tool boundary, not just
"sensitive" tools. There are no exempt tools.

---

## §G — Story Decomposition Seeds (Phase B-1 Input)

These six stories are the implementation mandate for ADR-022. They are seeds for Phase B-1
(story-writer dispatch); this appendix provides the scope contract so the story-writer can
produce full story specs without ambiguity.

---

### Story 1: S-WAVE5-PREP-01 — prism-bin Chassis

**Scope:** Create `crates/prism-bin/` as a new workspace member. Implement `main.rs`,
`cli.rs` (clap), `boot.rs` (boot sequence steps 1–11 calling stubs for steps 7/8/9/10 that
will be real by other stories), `signals.rs` (SIGTERM + SIGHUP), and `Cargo.toml` with
all required dependencies. The binary must compile and execute `prism start` (reaching the
"waiting for MCP clients" state once all stubs are filled by sibling stories). For Phase B-1,
a structural shell that wires steps 1–6 fully and steps 7–11 as `todo!()` marked with TD
annotations is acceptable as the initial partial-merge.

**Points estimate:** 5

**BC anchors:** BC-2.13.001 (server start), any Boot sequence BC created by story-writer for
boot-steps 1–6. New provisional BCs may be needed for: config-load (BC-2.BOOT.001), org-init
(BC-2.BOOT.002), credential-init (BC-2.BOOT.003), audit-init (BC-2.BOOT.004).

**Dependencies:** None (this is the root; other stories fill its stubs).

**Crates primarily touched:** `crates/prism-bin/` (new), `Cargo.toml` (workspace member add).

---

### Story 2: S-3.02-FOLLOWUP-RUNTIME — QueryEngine Execution Pipeline (HISTORICAL — merged PR #141 c6dd6602)

**Scope:** Implemented the `todo!()` sites in `prism-query`:
- `QueryEngine::execute` and `QueryEngine::execute_scheduled` in engine.rs
- `run_materialization_pipeline` and `resolve_source_refs` in materialization.rs
- `RocksDbTableProvider::{schema,table_type,scan,supports_filters_pushdown}` in internal_tables.rs
- `register_internal_tables` in internal_tables.rs

This story made the query engine functional end to end. Merged 2026-05-10 via PR #141 (c6dd6602).

**Points estimate:** 8

**BC anchors:** BC-2.11.001, BC-2.11.005, BC-2.11.006, BC-2.11.007, BC-2.11.011, BC-2.11.012,
BC-2.15.011.

**Dependencies:** S-WAVE5-PREP-01 (prism-bin exists so wiring can be tested end-to-end, but
the story can proceed in isolation with integration tests).

**Crates primarily touched:** `crates/prism-query/`.

---

### Story 3: W3-FIX-S307-001 — Sensor Adapter Write Overrides

**Scope:** Implement `fn write(...)` override in each of the four built-in sensor adapters:
CrowdStrike, Cyberint, Claroty, Armis. Each override must call the appropriate sensor write
API endpoint per the sensor's TOML spec `[[endpoints]]` write section. The default
`adapter.rs` body returns `WriteNotImplemented` and must not be replaced — it remains the
correct default for sensors that do not declare write endpoints.

**Points estimate:** 5

**BC anchors:** BC-2.04.007 (write operations contract), AD-022 (PrismQL Write Operations).

**Dependencies:** None — independent of other stories in this list.

**Crates primarily touched:** `crates/prism-sensors/` (adapter implementations per sensor).

---

### Story 4: W3-FIX-S307-002/003 — WriteExecutor Phase 3 + SQL DML

**Scope:**
- W3-FIX-S307-002: Wire `QueryMaterializer` into `WriteExecutor::execute` Phase 3 at
  `write_pipeline.rs` so it actually fetches records. Requires S-3.02-FOLLOWUP-RUNTIME
  to be merged first (materialization pipeline must be real to call it).
- W3-FIX-S307-003: Implement `WriteCapableTableProvider::{insert_into,delete_from,update}` at
  `write_table_registration.rs` to route SQL DML to `WriteExecutor`.

These can be a single story (combined scope ~5 points) or two stories (3+3). The story-writer
decides based on AC independence.

**Points estimate:** 5 (combined)

**BC anchors:** BC-2.04.007.

**Dependencies:** S-3.02-FOLLOWUP-RUNTIME (Phase 3 fetch calls materialization), W3-FIX-S307-001
(adapters must override write() or SQL DML has nothing to dispatch to).

**Crates primarily touched:** `crates/prism-query/` (write_pipeline.rs, write_table_registration.rs).

---

### Story 5: S-1.12-FOLLOWUP — Hot-Reload Watcher

**Scope:** Implement `HotReloadWatcher::{start,stop}` at `hot_reload.rs` per the
specification in §E of this ADR. Specifically: `notify` v7 integration, 500ms debounce,
dry-run validation gate before arc-swap, audit emission on reload success/fail, SIGHUP
handler integration, cross-platform quirk handling per §E table.

**Points estimate:** 3

**BC anchors:** BC-2.16.007 (hot reload contract).

**Dependencies:** S-WAVE5-PREP-01 (signal handler context for SIGHUP).

**Crates primarily touched:** `crates/prism-spec-engine/` (hot_reload.rs).

---

### Story 6: S-5.01-FOLLOWUP-MCP-BOOT — MCP Server Boot + Tool Registration

**Scope:** Implement the full `prism-mcp` crate per §F of this ADR:
- Add `rmcp 1.7` dependency to `prism-mcp/Cargo.toml` and workspace.
- Create `crates/prism-mcp/src/server.rs` with `PrismServer` struct and `serve_stdio`.
- Implement `#[tool_router]` for all tools declared in `api-surface.md` BC-2.13.* catalog.
- Wire injection defense (BC-2.09.001..008 scanner) at every tool entry boundary.
- Wire `ResponseEnvelope` wrapper on every tool response.
- Implement per-tool input validation + MCP error code mapping per §F table.

This story makes the runtime visible to Claude Code as an MCP server.

**Points estimate:** 8

**BC anchors:** BC-2.13.001..N (full tool catalog), BC-2.09.001..008 (injection defense),
BC-2.10.001..010 (MCP interface BCs).

**Dependencies:** S-WAVE5-PREP-01 (binary start invokes PrismServer::serve_stdio),
S-3.02-FOLLOWUP-RUNTIME (tool calls need a working query engine).

**Crates primarily touched:** `crates/prism-mcp/` (server.rs + tools/mod.rs + Cargo.toml).

---

## Alternatives Considered

### §D alt: RETIRE infusions

Rejected. See §D rationale — retirement cost exceeds REDO cost, and the feature has
meaningful differentiation value for MSSPs. Wasmtime 44 is already in the workspace.

### §A alt: Multiple binaries (prism-query-server + prism-mcp-server)

Rejected. Single-service deployment topology (ARCH-INDEX `deployment_topology: single-service`)
aligns with the per-analyst stdio model. Two binaries would require IPC, increase deployment
complexity, and conflict with the ephemeral-session usage pattern.

### §B alt: Parallel boot (all steps background)

Rejected. Steps 1–8 must be ordered because later steps depend on artifacts produced by
earlier steps (credentials needed by step 7 to open adapters; RocksDB needed by step 7
before QueryEngine in step 8 can register internal tables). Parallelism within a step is
fine (e.g., parallel sensor TOML validation) but step ordering is preserved.

### §F alt: TCP transport for MCP server

Rejected for MVP. stdio is the correct transport for Claude Code per-analyst deployment.
TCP transport can be added in a future story for multi-client or remote-server deployments,
but it introduces TLS, authentication, and connection management complexity out of scope.

---

## Source / Origin

Bundle B Phase B-0 architecture output. Authored at D-302 from workspace audit D-301
(F-AUD-D2-13, F-AUD-D1-01..06). Satisfies POL-15 for AD-005, AD-007, AD-018, AD-022.

---

## Related ADRs

| ADR | Relationship |
|-----|-------------|
| **ADR-023 §C4** (plugin runtime configuration) | Defines plugin-load step 7.5 intercalation between storage init (step 7) and query-engine init (step 8). ADR-023 §C4 is the authoritative specification for plugin-load step placement; it supersedes this ADR for that surface area per Source-of-Truth Precedence Rule 2. |
| **ADR-021** | BC/VP promotion lifecycle — boot steps interact with lifecycle state transitions |
| **ADR-020** | Story status taxonomy — story-seed numbering in §G |
| **ADR-005** | rmcp MCP server — wired at step 9 |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.17 | 2026-07-13 | architect | F-MCPNULL-P4-OBS-002 doc-accuracy correction (DEFECT-MCP-ROWSHAPE-NULLS-001 pass-4, orchestrator adjudication): §F Error-code mapping table `PrismError::InternalError` row corrected from `-32603` to `-32000`. Verification: `rg -- '-32603' crates/` returns zero hits; `codes::INTERNAL_ERROR = -32000` is the shipped constant in `prism-mcp/src/error_mapping.rs` since PR #163. Server-defined range chosen because `-32603` is the transport-level JSON-RPC internal-error code reserved by the rmcp layer; application-level redacted internal errors use the server-defined `-32000` range. Brief rationale note added below the table. Redaction posture unchanged. Doc-accuracy correction only — no design change. |
| 1.16 | 2026-07-13 | architect | F-MCPNULL-P1-HIGH-001 (DEFECT-MCP-ROWSHAPE-NULLS-001) closure: §F Error-code mapping table updated for BC-2.10.007 [H8b] message/suggestion split — `PrismError::InternalError` message updated from `"Internal error; see audit log"` to `"Internal error"` (message field) + `"See audit log for details."` (suggestion field); suggestion column added to table (other rows set to `—` — no suggestion defined). Redaction posture unchanged: internal errors remain opaque to the caller. ADR-038 §Context line citing this string is a historical description of a pre-fix defective state (exempt per TD-VSDD-091). |
| 1.15 | 2026-06-10 | architect | BOOT-02 closure (2026-06-10 full-codebase review package, human-approved): §B step 7 column-family count corrected 17→19 to match code source of truth (`prism-core/src/storage.rs` `ALL_DOMAINS: [StorageDomain; 19]` — 16 S-1.01 domains + 3 S-1.02 domains `credentials`/`feature_flags`/`scheduler`; rocksdb_backend opens and health-checks all 19). Stale "17 (per AD-004)" reflected the pre-S-1.02 AD-004 enumeration, which also counted the not-yet-implemented `case_dedup_idx` CF (S-4.06 Task 9b, planned). AD-004 row in ARCH-INDEX, data-layer.md §Persistent Data Path, and system-overview.md platform-layer diagram corrected in the same burst (TD-VSDD-060 sibling-site sweep). |
| 1.14 | 2026-05-31 | architect | S-DEMO-001 boot step 9A wiring accuracy (GAP-002-A closure). §B step 7: replaced stale `AdapterRegistry::init_registry_for_org` action with `AdapterRegistry::new()` (empty) and note that population happens at step 9A. §B: added step 7.5b (PluginAuthProvider construction via validate_and_construct_auth_providers, plugin_auth_providers threaded to step 9) and step 7.5c (dynamic write-tool registration) sub-steps. §B step 9: added step 9A sub-step documenting step9a_populate_adapter_registry call (called from within step9_start_mcp_server before QueryEngine construction) per BC-2.22.001 §Step 9A. §C QueryEngine contract: added note that AdapterRegistry is populated by step9a_populate_adapter_registry before QueryEngine::new() (S-DEMO-001). No architectural redesign — wiring-only accuracy corrections per TD-VSDD-091. |
| 1.13 | 2026-05-28 | implementer | F-PASS9-MED-1 closure: all 7 "rmcp 1.4" narrative references updated to "rmcp 1.7" (frontmatter runtime_deliverables, §Decision, §F transport note, §B Step 9, §F heading, §F inline dependency sentence, §G Story 6 scope). OQ-1 confirmed: rmcp 1.4 unavailable on crates.io; 1.7 is the actual published version used at TDD time. ARCH-INDEX AD-005 row updated in same burst (F-PASS9-MED-1). Version 1.12→1.13. |
| 1.12 | 2026-05-17 | architect | FB73 F-LP85-HIGH-001 closure (architect scope): ADR-026 §D7 v1.22→v1.23 at line 243 §B Step 8 first-statement note (1 site). 7th 1-finding cascade-restart-#4 attempt — cross-value-class side-effect dimension. PO swept 7 spec files in same burst. POL-29 v1.25→v1.26 step 8g by SM. Sibling-sweep other ADRs: 0 additional sites found. |
| 1.11 | 2026-05-17 | architect | FB69 F-LP81-HIGH-002 closure (architect scope): ADR-026 §D7 pin v1.21→v1.22 at line 243 §B Step 8 first-statement note (1 site). 22nd+ recurrence of POL-29 step 3a class (b) — META-META gap revealed: FB62 SM step 8b iteration bumped ADR-026 v1.21→v1.22 but didn't trigger own external-cite sweep. POL-29 v1.22→v1.23 step 8d META-META transitive closure by state-manager. PO swept 6 spec files in same burst. POL-29 step 8c grep evidence (architecture-domain): variant 1 pre=0 post=0, variant 2 pre=1 post=0, variant 3 pre=0 post=0, variant 4 pre=0 post=0. Sibling-sweep across other ADRs: 0 additional sites found. |
| 1.10 | 2026-05-17 | architect | F-LP74-HIGH-001 closure (architect scope): ADR-026 §D7 pin v1.19→v1.21 at line 243 §B Step 8 first-statement note (1 site). Recurrence #20 of POL-29 step 3a registry class (b); META-gap revealed by pass-74 — POL-29 v1.17 step 8a single-pass enforcement misses transitively-introduced staleness within own application cycle. PO swept 6 spec files (story v1.38 + BC-2.16.012 v1.23 + BC-2.16.002 v1.27 + error-taxonomy v1.35 + VP-156 v0.15 + HS-003 v1.12) in same burst. POL-29 v1.17→v1.18 step 8b transitive closure amendment by state-manager (in-burst META-gap closure per user strategic direction). POL-29 step 8a grep evidence (architecture-domain): pre-grep 1 → post-grep 0. Sibling-sweep across other ADRs: 0 additional sites found. |
| 1.9 | 2026-05-17 | architect | F-LP71-HIGH-001 sibling-sweep catch: H1 extended to byte-match frontmatter `title:` — `# ADR-022: Production Runtime Wiring` → `# ADR-022: Production Runtime Wiring — prism-bin Chassis, Boot Sequence, Wiring Contracts, Infusion Fate, Hot-Reload Watcher, MCP Topology`. ARCH-INDEX row already has the long form (canonical); H1 was the truncated outlier. POL-7 + TD-VSDD-060 within-file frontmatter↔H1 dimension. ARCH-INDEX row propagation (version bump v1.8→v1.9) owned by state-manager. |
| 1.8 | 2026-05-17 | architect | F-LP68-HIGH-001 closure cascade (FB56b architect scope): ADR-026 §D7 pin v1.18→v1.19 propagation at ADR-022 §B Step 8 first-statement note (line 243; 1 site). Triggered by FB56 architect bump of ADR-026 v1.18→v1.19 to close 1 error-taxonomy cite at line 312; POL-29 v1.17 step 8a FIRST APPLICATION CATCH — diff-derived value-class enumeration detected the side-effect D7 v1.18 staleness BEFORE commit. PO swept story v1.35 + BC-2.16.012 v1.22 + VP-156 v0.14 + HS-003 v1.10 + error-taxonomy v1.34 + BC-2.16.002 v1.26 in same burst. POL-29 v1.17 step 8a grep evidence (architecture-domain): pre-grep 1 → post-grep 0. Sibling-sweep across other ADRs: 0 found. |
| 1.7 | 2026-05-17 | architect | F-LP67-HIGH-001 closure (architect scope): ADR-026 §D7 pin v1.17→v1.18 propagation at ADR-022 §B Step 8 first-statement note (line 243; 1 live-narrative site). POL-29 v1.16 step 3a (b) ADR-026 D7 pin registry first-test surfaced recurrence #18 of class (b); PO swept BC-2.16.012 v1.21 + VP-156 v0.13 + HS-003 v1.9 + error-taxonomy v1.33 + story v1.33 in same burst. POL-29 step 8 STRENGTHENED grep evidence (architect-domain): pre-grep 1 → post-grep 0 in `.factory/specs/architecture/`. Sibling-sweep across other ADRs: 0 additional sites found. |
| 1.6 | 2026-05-17 | state-manager | FB51 F-LP63-HIGH-002 closure: §Changelog row positions repaired to strict descending (v1.3 moved below v1.4; v1.5 moved above v1.4); 6th POL-26 monotonic-ordering recurrence repair per D-611/D-628/D-635/D-659/D-670/D-671 precedent. POL-26 corollary: row content immutable; position is bookkeeping. |
| 1.5 | 2026-05-17 | architect | FB50 POL-23 sibling-sweep OBS-LP62-002 interpretation #2: §B Step 8 live-narrative ADR-026 §D7 v1.16 pin bumped to v1.17 (current ADR-026 version per FB47 §Related ADRs row edit; D7 content unchanged since v1.16). |
| 1.4 | 2026-05-16 | architect | FB45: §B Step 8 description: append first-statement note — `prism_query::invalidation::mark_query_phase_started()` is the first statement of step-8, closing the write-registration window before `QueryEngine::new()`. Cross-references ADR-026 §D7 v1.16. Closes OBS-LP57-001 Path A per Canonical Principle Rule 4. |
| 1.3 | 2026-05-13 | architect | Closes F-LP8-OBS-001 (PREREQ-D fix-burst-7 stage 1B): add step 7.5 plugin-load cross-reference to §B boot sequence table and Related ADRs section; update traffic-gate note to include step 7.5; bump version. ADR-023 §C4 is cited as the authoritative placement spec; Source-of-Truth Precedence Rule 2 noted inline. No architectural content changed — editorial discoverability amendment only. |
| 1.2 | 2026-05-12 | state-manager | TD-VSDD-091 volatile-pin strip per audit at cycles/wave-4-operations/sprint-review-PREREQ-trio.md §7. No architectural content change. 18 line-number citations stripped across §Context/§B/§C/§D/§G; function-name pivots applied for InfusionLoader::{parse,load_all,validate_credentials}, InfusionLruCache::{get,insert}, MmdbSource::{load,enrich_single,enrich_batch}, plugin_bridge::enrich_via_plugin, QueryEngine::execute and execute_scheduled, RocksDbTableProvider::{schema,scan,...}, register_internal_tables. engine.rs/materialization.rs/internal_tables.rs references marked HISTORICAL post S-3.02-FOLLOWUP-RUNTIME merge c6dd6602. Added missing template H2 sections (Decision, Rationale, Consequences, Source / Origin) per template compliance. |
| 1.1 | 2026-05-09 | product-owner | §B step 2: replace stale `~/.prism/` literal with platform-aware default to match BC-2.06.011 v1.2 phrasing. Closes F-P6-MED-1 from PR #139 PR-LEVEL adversary pass-6. |
| 1.0 | 2026-05-08 | architect | Initial authorship — Bundle B Phase B-0 architecture output |
