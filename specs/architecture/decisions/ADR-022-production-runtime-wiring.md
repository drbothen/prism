---
document_type: adr
adr_id: "ADR-022"
title: "Production Runtime Wiring — prism-bin Chassis, Boot Sequence, Wiring Contracts, Infusion Fate, Hot-Reload Watcher, MCP Topology"
status: ACCEPTED
date: "2026-05-08"
version: "1.1"
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
  - crates/prism-mcp/src/server.rs         # rmcp 1.4 PrismServer struct
  - crates/prism-mcp/src/tools/mod.rs      # tool router + all 35+ tool implementations
  - crates/prism-spec-engine/src/hot_reload.rs  # HotReloadWatcher::start/stop (notify 7)
wiring_deferred_to: null  # This ADR IS the wiring specification — no further deferral
input-hash: "[md5]"
---

# ADR-022: Production Runtime Wiring

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
- `QueryEngine::execute` at `engine.rs:276` is `todo!("S-3.02 — QueryEngine::execute")`.
- `run_materialization_pipeline` at `materialization.rs:241` is `todo!("S-3.02 — ...")`.
- `RocksDbTableProvider::schema/scan/register_internal_tables` at `internal_tables.rs:125/139/168`
  are `todo!("S-3.02 — ...")`.
- `WriteExecutor::execute` Phase 3 fetch at `write_pipeline.rs:349` is `let fetched_records:
  Vec<...> = vec![];` — hardcoded empty, never fetches records.
- `WriteCapableTableProvider::insert_into/delete_from/update` at
  `write_table_registration.rs:176/190/205` return `DataFusionError::NotImplemented("S-3.07-pending")`.
- `SensorAdapter::write()` default at `adapter.rs:365` returns `WriteNotImplemented` for all
  four built-in sensors; no concrete override exists.
- `HotReloadWatcher::start/stop` at `hot_reload.rs:66/72` are
  `unimplemented!("S-1.12: ... Red Gate stub")`.
- `ConfigManager::new` and `parse_spec_directory` are called only from test code; no production
  binary instantiates them (verified: grep for callers outside test files returns zero matches).

Bundles A (taxonomy reform), B (runtime gap), D (doc cleanup) are the three cleanup epics
approved at D-302. This ADR is the Phase B-0 architecture output for Bundle B.

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

The `start` subcommand connects the rmcp 1.4 server to stdio transport. This is the only
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
         NOTE: currently parse_spec_directory (prism-spec-engine/src/config_manager.rs:75)
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
         Action: open RocksDB with all 17 column families (per AD-004; prism-storage)
         Action: call register_internal_tables (prism-query/src/internal_tables.rs:164)
                 — currently todo!("S-3.02 — register_internal_tables"); contract gap
         Action: construct AdapterRegistry::init_registry_for_org per loaded sensor specs
         Failure: exit 4 (RocksDB open failure or internal-tables registration failure)

Step 8   [BLOCKING → BACKGROUND] QueryEngine + WriteExecutor construction
         Action: construct QueryEngine (prism-query); bind AdapterRegistry + StorageBackend
         Action: construct WriteExecutor (prism-query); bind feature-flag check + capability check
         Note: QueryEngine::execute at engine.rs:276 is todo!() — S-3.02-FOLLOWUP-RUNTIME
               resolves this before Step 8 can function
         After construction completes: engine accepts queries (via MCP tools)
         Failure: exit 4

Step 9   [BACKGROUND] MCP server start
         Action: call PrismServer::new(engine, write_executor, audit_emitter, security_config)
         Action: bind rmcp 1.4 stdio transport
         Action: register all tools via #[tool_router] macro (§F tool inventory)
         Action: enable prompt-injection defense middleware (§F, BC-2.09.001..008)
         Once: write "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}" to stdout
         MCP server now accepting tool calls
         Failure: log error + exit 4

Step 10  [BACKGROUND] Hot-reload watcher install
         Action: call HotReloadWatcher::start(manager.clone(), config.spec_dir, debounce_ms=500)
         Action: currently unimplemented!() at hot_reload.rs:66 — S-1.12-FOLLOWUP resolves
         Background task: fs events → validate → arc-swap (§E)
         Failure: log warning (non-fatal — degrade gracefully without reload; alert is emitted)

Step 11  [BACKGROUND] Signal handler install
         Action: register tokio signal handlers for SIGTERM + SIGHUP
         SIGTERM: initiate graceful shutdown (drain in-flight queries → close MCP server →
                  flush audit buffer → close RocksDB → exit 0)
         SIGHUP: trigger manual config reload (same code path as hot-reload watcher)
         Failure: log error, continue (OS may still deliver signals)
```

**Traffic gate:** steps 1–8 are blocking. Queries cannot reach `QueryEngine::execute` until
step 8 completes. The MCP server (step 9) only starts after step 8. Steps 10–11 are
background and non-fatal; their failure degrades capability but does not prevent serving queries.

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

// File: crates/prism-spec-engine/src/config_manager.rs:75 — REAL (not stubbed)
pub fn parse_spec_directory(spec_dir: &Path) -> Result<ConfigSnapshot, SpecEngineError>;

// File: crates/prism-spec-engine/src/config_manager.rs:27 — REAL (not stubbed)
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
// Boot step 8 call site
use prism_query::engine::QueryEngine;

// File: crates/prism-query/src/engine.rs — constructor is REAL
impl QueryEngine {
    pub fn new(registry: Arc<AdapterRegistry>, storage: Arc<dyn RocksStorageBackend>,
               ocsf: Arc<OcsfNormalizer>) -> Self;
    // execute is TODO — contract gap:
    pub async fn execute(&self, query_str: &str, options: QueryOptions)
        -> Result<QueryResult, PrismError>;  // engine.rs:276 — todo!("S-3.02")
}
```

Contract gap: `QueryEngine::execute` at `crates/prism-query/src/engine.rs:276` is `todo!()`.
Also: `run_materialization_pipeline` at `materialization.rs:241` and `resolve_source_refs` at
`materialization.rs:263` are `todo!()`. `RocksDbTableProvider::schema/scan` at
`internal_tables.rs:125/139` are `todo!()`. `register_internal_tables` at
`internal_tables.rs:168` is `todo!()`. All are resolved by `S-3.02-FOLLOWUP-RUNTIME`.

### WriteExecutor (SS-11 / prism-query)

```rust
// Boot step 8 call site
use prism_query::write_pipeline::WriteExecutor;

// Constructor: REAL (not stubbed)
impl WriteExecutor {
    pub fn new(feature_flags: Arc<FeatureFlagStore>, audit: Arc<AuditEmitter>) -> Self;
    // execute has structural gap — Phase 3 fetch hardcoded empty:
    pub async fn execute(&self, plan: WritePlan)
        -> Result<WriteExecutionReport, PrismError>;  // write_pipeline.rs:349 — empty vec![]
}
```

Contract gaps:
- `write_pipeline.rs:349` — Phase 3 fetch returns `vec![]` (never fetches records).
- `adapter.rs:365` — `SensorAdapter::write()` default returns `WriteNotImplemented`; no
  concrete override exists for CrowdStrike, Cyberint, Claroty, or Armis.
- `write_table_registration.rs:176/190/205` — `insert_into/delete_from/update` return
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
                 debounce_ms: u64) -> Result<(), SpecEngineError>;  // hot_reload.rs:66
    pub fn stop(&self) -> Result<(), SpecEngineError>;               // hot_reload.rs:72
}
```

Contract gap: both methods are `unimplemented!()` at `hot_reload.rs:66/72`. Resolved by
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
1. `InfusionLoader::parse/load_all/validate_credentials` (loader.rs:42/50/57/66)
2. `InfusionLruCache::get/insert` (cache.rs:109/120) — LRU backed by RocksDB CF `infusion_cache`
3. `MmdbSource::load/enrich_single/enrich_batch` (sources/mmdb.rs:23/29/37)
4. `CsvSource` and `JsonLookupSource` equivalents
5. `plugin_bridge::enrich_via_plugin` (plugin_bridge.rs:26/37) — calls S-1.15 WASM runtime
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

### rmcp 1.4 integration

`prism-mcp` gains a dependency on `rmcp` version 1.4 (per AD-005). This must be added to
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

All tools are registered in a single `#[tool_router]` impl block per the rmcp 1.4 API.
The 35-tool claim in `module-decomposition.md:513` is aspirational but grounded in BC-2.13.*
(the tool catalog BC). Tool count at MVP target: the full BC-2.13.* catalog.

### Per-tool input validation

Each tool handler MUST:
1. Deserialize the MCP tool parameters from JSON.
2. Validate required fields (non-null, type correct, within bounds).
3. On validation failure: return a structured MCP error response (`code: -32602`,
   message describing the invalid field). NEVER `panic!()` or `unwrap()`.

### Error-code mapping

| PrismError variant | MCP error code | message |
|---|---|---|
| `ParseError` | -32602 (Invalid params) | "PrismQL parse error: {detail}" |
| `SensorError::WriteNotImplemented` | -32003 (Custom) | "Write not supported for sensor: {sensor}" |
| `PrismError::PermissionDenied` | -32002 (Custom) | "Feature flag denied: {flag}" |
| `PrismError::Timeout` | -32001 (Custom) | "Query timeout exceeded" |
| `PrismError::InternalError` | -32603 (Internal error) | "Internal error; see audit log" |

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

### Story 2: S-3.02-FOLLOWUP-RUNTIME — QueryEngine Execution Pipeline

**Scope:** Implement the eight `todo!()` sites in `prism-query`:
- `QueryEngine::execute` at `engine.rs:276`
- `QueryEngine::execute_scheduled` at `engine.rs:317`
- `run_materialization_pipeline` at `materialization.rs:241`
- `resolve_source_refs` at `materialization.rs:263`
- `RocksDbTableProvider::schema/table_type/scan/supports_filters_pushdown` at
  `internal_tables.rs:125/129/139/146`
- `register_internal_tables` at `internal_tables.rs:168`

This story makes the query engine functional from end to end. It is the critical-path
dependency for all MCP tool calls that execute PrismQL.

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
`adapter.rs:365` body returns `WriteNotImplemented` and must not be replaced — it remains the
correct default for sensors that do not declare write endpoints.

**Points estimate:** 5

**BC anchors:** BC-2.04.007 (write operations contract), AD-022 (PrismQL Write Operations).

**Dependencies:** None — independent of other stories in this list.

**Crates primarily touched:** `crates/prism-sensors/` (adapter implementations per sensor).

---

### Story 4: W3-FIX-S307-002/003 — WriteExecutor Phase 3 + SQL DML

**Scope:**
- W3-FIX-S307-002: Wire `QueryMaterializer` into `WriteExecutor::execute` Phase 3 at
  `write_pipeline.rs:349` so it actually fetches records. Requires S-3.02-FOLLOWUP-RUNTIME
  to be merged first (materialization pipeline must be real to call it).
- W3-FIX-S307-003: Implement `WriteCapableTableProvider::insert_into/delete_from/update` at
  `write_table_registration.rs:176/190/205` to route SQL DML to `WriteExecutor`.

These can be a single story (combined scope ~5 points) or two stories (3+3). The story-writer
decides based on AC independence.

**Points estimate:** 5 (combined)

**BC anchors:** BC-2.04.007.

**Dependencies:** S-3.02-FOLLOWUP-RUNTIME (Phase 3 fetch calls materialization), W3-FIX-S307-001
(adapters must override write() or SQL DML has nothing to dispatch to).

**Crates primarily touched:** `crates/prism-query/` (write_pipeline.rs, write_table_registration.rs).

---

### Story 5: S-1.12-FOLLOWUP — Hot-Reload Watcher

**Scope:** Implement `HotReloadWatcher::start/stop` at `hot_reload.rs:66/72` per the
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
- Add `rmcp 1.4` dependency to `prism-mcp/Cargo.toml` and workspace.
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

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-08 | architect | Initial authorship — Bundle B Phase B-0 architecture output |
| 1.1 | 2026-05-09 | product-owner | §B step 2: replace stale `~/.prism/` literal with platform-aware default to match BC-2.06.011 v1.2 phrasing. Closes F-P6-MED-1 from PR #139 PR-LEVEL adversary pass-6. |
